/**
 * Collaborative Context Manager - Team context sharing and synchronization
 * 
 * This system enables teams to share AI context, collaborate on code understanding,
 * and maintain synchronized context across team members.
 */

import { AIContextCheckpoint, CompleteAIContext, QueryIntent } from '../AIContextBuilder';
import { contextCache } from '../cache/ContextCache';

export class CollaborativeContextManager {
    private teamManager: TeamManager;
    private contextSyncService: ContextSyncService;
    private permissionManager: PermissionManager;
    private sharedContexts: Map<string, SharedContextPackage> = new Map();
    private config: CollaborativeConfig;

    constructor(config: CollaborativeConfig = {}) {
        this.config = {
            maxSharedContexts: config.maxSharedContexts || 100,
            syncIntervalMs: config.syncIntervalMs || 30000,
            maxTeamSize: config.maxTeamSize || 50,
            retentionDays: config.retentionDays || 30,
            enableRealTimeSync: config.enableRealTimeSync !== false,
            encryptSharedData: config.encryptSharedData !== false
        };

        this.teamManager = new TeamManager(this.config);
        this.contextSyncService = new ContextSyncService(this.config);
        this.permissionManager = new PermissionManager();
    }

    /**
     * Share context with team members
     */
    async shareContextWithTeam(
        checkpoint: AIContextCheckpoint,
        teamId: string,
        sharedBy: string,
        options: ShareOptions = {}
    ): Promise<SharedContextPackage> {
        try {
            // Get team information
            const team = await this.teamManager.getTeam(teamId);
            if (!team) {
                throw new Error(`Team not found: ${teamId}`);
            }

            // Check permissions
            await this.permissionManager.checkSharePermission(sharedBy, teamId);

            // Create personalized context views for each team member
            const personalizedViews = await this.createPersonalizedViews(
                checkpoint,
                team.members,
                options
            );

            // Extract common context useful for everyone
            const commonContext = await this.extractCommonContext(checkpoint);

            // Create role-specific context
            const roleSpecificContext = await this.createRoleSpecificContext(
                checkpoint,
                team.members
            );

            // Create shared context package
            const sharedPackage: SharedContextPackage = {
                id: this.generateShareId(),
                checkpoint_id: checkpoint.id,
                team_id: teamId,
                shared_by: sharedBy,
                common_context: commonContext,
                personalized_views: personalizedViews,
                role_specific_context: roleSpecificContext,
                metadata: {
                    created_at: new Date(),
                    expires_at: new Date(Date.now() + (this.config.retentionDays ?? 30) * 24 * 60 * 60 * 1000),
                    access_count: 0,
                    last_accessed: new Date(),
                    tags: options.tags || [],
                    description: options.description
                },
                permissions: {
                    read: team.members.map(m => m.id),
                    write: options.allowEdit ? [sharedBy] : [],
                    admin: [sharedBy]
                }
            };

            // Encrypt if enabled
            if (this.config.encryptSharedData) {
                sharedPackage.encrypted = true;
                sharedPackage.common_context = await this.encryptData(sharedPackage.common_context);
            }

            // Store shared context
            this.sharedContexts.set(sharedPackage.id, sharedPackage);

            // Sync with team members
            if (this.config.enableRealTimeSync) {
                await this.contextSyncService.syncToTeam(sharedPackage, team.members);
            }

            // Notify team members
            await this.notifyTeamMembers(sharedPackage, team.members, sharedBy);

            return sharedPackage;

        } catch (error) {
            console.error('Failed to share context with team:', error);
            throw new Error(`Context sharing failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Get shared context for a team member
     */
    async getSharedContext(
        shareId: string,
        userId: string,
        teamId: string
    ): Promise<PersonalizedSharedContext | null> {
        try {
            const sharedPackage = this.sharedContexts.get(shareId);
            if (!sharedPackage) {
                return null;
            }

            // Check permissions
            if (!await this.permissionManager.checkReadPermission(userId, sharedPackage)) {
                throw new Error('Access denied');
            }

            // Check expiration
            if (sharedPackage.metadata.expires_at < new Date()) {
                this.sharedContexts.delete(shareId);
                return null;
            }

            // Get personalized view for this user
            const personalizedView = sharedPackage.personalized_views.find(
                view => view.member_id === userId
            );

            // Get role-specific context
            const userRole = await this.teamManager.getUserRole(userId, teamId);
            const roleContext = sharedPackage.role_specific_context[userRole] || {};

            // Update access tracking
            sharedPackage.metadata.access_count++;
            sharedPackage.metadata.last_accessed = new Date();

            // Decrypt if needed
            let commonContext = sharedPackage.common_context;
            if (sharedPackage.encrypted) {
                commonContext = await this.decryptData(commonContext);
            }

            return {
                id: shareId,
                common_context: commonContext,
                personalized_context: personalizedView?.context || {},
                role_context: roleContext,
                metadata: sharedPackage.metadata,
                shared_by: sharedPackage.shared_by
            };

        } catch (error) {
            console.error('Failed to get shared context:', error);
            return null;
        }
    }

    /**
     * Update shared context with collaborative edits
     */
    async updateSharedContext(
        shareId: string,
        updates: ContextUpdate[],
        userId: string
    ): Promise<boolean> {
        try {
            const sharedPackage = this.sharedContexts.get(shareId);
            if (!sharedPackage) {
                return false;
            }

            // Check write permissions
            if (!await this.permissionManager.checkWritePermission(userId, sharedPackage)) {
                throw new Error('Write access denied');
            }

            // Apply updates
            for (const update of updates) {
                await this.applyContextUpdate(sharedPackage, update, userId);
            }

            // Sync changes to team
            if (this.config.enableRealTimeSync) {
                const team = await this.teamManager.getTeam(sharedPackage.team_id);
                if (team) {
                    await this.contextSyncService.syncUpdates(sharedPackage, updates, team.members);
                }
            }

            return true;

        } catch (error) {
            console.error('Failed to update shared context:', error);
            return false;
        }
    }

    /**
     * Search shared contexts within a team
     */
    async searchSharedContexts(
        teamId: string,
        query: string,
        userId: string,
        filters: SearchFilters = {}
    ): Promise<SharedContextSearchResult[]> {
        const results: SharedContextSearchResult[] = [];

        for (const [shareId, sharedPackage] of this.sharedContexts) {
            // Check team membership
            if (sharedPackage.team_id !== teamId) {
                continue;
            }

            // Check permissions
            if (!await this.permissionManager.checkReadPermission(userId, sharedPackage)) {
                continue;
            }

            // Apply filters
            if (!this.matchesFilters(sharedPackage, filters)) {
                continue;
            }

            // Calculate relevance score
            const relevanceScore = this.calculateSearchRelevance(sharedPackage, query);
            if (relevanceScore > 0.3) {
                results.push({
                    id: shareId,
                    title: this.generateContextTitle(sharedPackage),
                    description: sharedPackage.metadata.description || '',
                    shared_by: sharedPackage.shared_by,
                    created_at: sharedPackage.metadata.created_at,
                    relevance_score: relevanceScore,
                    tags: sharedPackage.metadata.tags,
                    access_count: sharedPackage.metadata.access_count
                });
            }
        }

        return results.sort((a, b) => b.relevance_score - a.relevance_score);
    }

    /**
     * Get collaborative analytics for a team
     */
    async getCollaborativeAnalytics(teamId: string, userId: string): Promise<CollaborativeAnalytics> {
        // Check admin permissions
        const team = await this.teamManager.getTeam(teamId);
        if (!team || !await this.permissionManager.checkAdminPermission(userId, teamId)) {
            throw new Error('Admin access required');
        }

        const teamSharedContexts = Array.from(this.sharedContexts.values())
            .filter(pkg => pkg.team_id === teamId);

        const totalShared = teamSharedContexts.length;
        const totalAccesses = teamSharedContexts.reduce((sum, pkg) => sum + pkg.metadata.access_count, 0);
        
        // Calculate sharing patterns
        const sharingPatterns = this.analyzeSharingPatterns(teamSharedContexts);
        
        // Calculate collaboration metrics
        const collaborationMetrics = this.calculateCollaborationMetrics(teamSharedContexts, team.members);

        return {
            team_id: teamId,
            total_shared_contexts: totalShared,
            total_accesses: totalAccesses,
            average_accesses_per_context: totalShared > 0 ? totalAccesses / totalShared : 0,
            active_collaborators: this.countActiveCollaborators(teamSharedContexts),
            sharing_patterns: sharingPatterns,
            collaboration_metrics: collaborationMetrics,
            generated_at: new Date()
        };
    }

    // Helper methods

    private async createPersonalizedViews(
        checkpoint: AIContextCheckpoint,
        members: TeamMember[],
        options: ShareOptions
    ): Promise<PersonalizedView[]> {
        const views: PersonalizedView[] = [];

        for (const member of members) {
            const personalizedContext = await this.createPersonalizedContext(checkpoint, member);
            const accessLevel = this.determineAccessLevel(member, options);

            views.push({
                member_id: member.id,
                role: member.role,
                context: personalizedContext,
                access_level: accessLevel,
                customizations: await this.getUserCustomizations(member.id)
            });
        }

        return views;
    }

    private async createPersonalizedContext(
        checkpoint: AIContextCheckpoint,
        member: TeamMember
    ): Promise<any> {
        // Create context tailored to the member's role and preferences
        const baseContext = checkpoint.semantic_context;
        
        // Filter based on role
        switch (member.role) {
            case 'developer':
                return this.filterForDeveloper(baseContext);
            case 'architect':
                return this.filterForArchitect(baseContext);
            case 'tester':
                return this.filterForTester(baseContext);
            case 'manager':
                return this.filterForManager(baseContext);
            default:
                return baseContext;
        }
    }

    private async extractCommonContext(checkpoint: AIContextCheckpoint): Promise<any> {
        // Extract context that's useful for all team members
        return {
            project_overview: this.extractProjectOverview(checkpoint),
            key_components: this.extractKeyComponents(checkpoint),
            architectural_decisions: checkpoint.intent_analysis?.architectural_decisions || [],
            common_patterns: this.extractCommonPatterns(checkpoint)
        };
    }

    private async createRoleSpecificContext(
        checkpoint: AIContextCheckpoint,
        members: TeamMember[]
    ): Promise<{ [role: string]: any }> {
        const roleContext: { [role: string]: any } = {};
        const roles = [...new Set(members.map(m => m.role))];

        for (const role of roles) {
            switch (role) {
                case 'developer':
                    roleContext[role] = {
                        implementation_details: this.extractImplementationDetails(checkpoint),
                        code_examples: this.extractCodeExamples(checkpoint),
                        debugging_info: this.extractDebuggingInfo(checkpoint)
                    };
                    break;
                    
                case 'architect':
                    roleContext[role] = {
                        system_design: this.extractSystemDesign(checkpoint),
                        design_patterns: checkpoint.intent_analysis?.design_patterns_used || [],
                        scalability_considerations: this.extractScalabilityInfo(checkpoint)
                    };
                    break;
                    
                case 'tester':
                    roleContext[role] = {
                        test_scenarios: this.extractTestScenarios(checkpoint),
                        edge_cases: this.extractEdgeCases(checkpoint),
                        quality_metrics: this.extractQualityMetrics(checkpoint)
                    };
                    break;
                    
                case 'manager':
                    roleContext[role] = {
                        project_impact: this.extractProjectImpact(checkpoint),
                        timeline_implications: this.extractTimelineInfo(checkpoint),
                        resource_requirements: this.extractResourceInfo(checkpoint)
                    };
                    break;
            }
        }

        return roleContext;
    }

    private determineAccessLevel(member: TeamMember, options: ShareOptions): AccessLevel {
        if (options.restrictedAccess && !options.allowedUsers?.includes(member.id)) {
            return 'none';
        }
        
        switch (member.role) {
            case 'manager':
            case 'architect':
                return 'full';
            case 'developer':
                return 'standard';
            case 'tester':
                return 'limited';
            default:
                return 'limited';
        }
    }

    private async notifyTeamMembers(
        sharedPackage: SharedContextPackage,
        members: TeamMember[],
        sharedBy: string
    ): Promise<void> {
        const notification = {
            type: 'context_shared',
            share_id: sharedPackage.id,
            shared_by: sharedBy,
            title: this.generateContextTitle(sharedPackage),
            timestamp: new Date()
        };

        // Send notifications (implementation would depend on notification system)
        for (const member of members) {
            if (member.id !== sharedBy) {
                console.log(`Notifying ${member.id} about shared context: ${notification.title}`);
            }
        }
    }

    private generateShareId(): string {
        return `share_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    private generateContextTitle(sharedPackage: SharedContextPackage): string {
        // Generate a meaningful title from the context
        const description = sharedPackage.metadata.description;
        if (description) {
            return description;
        }
        
        return `Shared Context - ${sharedPackage.metadata.created_at.toLocaleDateString()}`;
    }

    private async encryptData(data: any): Promise<any> {
        // Placeholder for encryption
        return { encrypted: true, data: JSON.stringify(data) };
    }

    private async decryptData(encryptedData: any): Promise<any> {
        // Placeholder for decryption
        if (encryptedData.encrypted) {
            return JSON.parse(encryptedData.data);
        }
        return encryptedData;
    }

    private async applyContextUpdate(
        sharedPackage: SharedContextPackage,
        update: ContextUpdate,
        userId: string
    ): Promise<void> {
        // Apply the update based on its type
        switch (update.type) {
            case 'annotation':
                await this.addAnnotation(sharedPackage, update, userId);
                break;
            case 'highlight':
                await this.addHighlight(sharedPackage, update, userId);
                break;
            case 'comment':
                await this.addComment(sharedPackage, update, userId);
                break;
        }
    }

    private async addAnnotation(sharedPackage: SharedContextPackage, update: ContextUpdate, userId: string): Promise<void> {
        // Add annotation logic
    }

    private async addHighlight(sharedPackage: SharedContextPackage, update: ContextUpdate, userId: string): Promise<void> {
        // Add highlight logic
    }

    private async addComment(sharedPackage: SharedContextPackage, update: ContextUpdate, userId: string): Promise<void> {
        // Add comment logic
    }

    private matchesFilters(sharedPackage: SharedContextPackage, filters: SearchFilters): boolean {
        if (filters.tags && filters.tags.length > 0) {
            if (!filters.tags.some(tag => sharedPackage.metadata.tags.includes(tag))) {
                return false;
            }
        }

        if (filters.sharedBy && sharedPackage.shared_by !== filters.sharedBy) {
            return false;
        }

        if (filters.dateRange) {
            const createdAt = sharedPackage.metadata.created_at.getTime();
            if (filters.dateRange.start && createdAt < filters.dateRange.start.getTime()) {
                return false;
            }
            if (filters.dateRange.end && createdAt > filters.dateRange.end.getTime()) {
                return false;
            }
        }

        return true;
    }

    private calculateSearchRelevance(sharedPackage: SharedContextPackage, query: string): number {
        const queryLower = query.toLowerCase();
        let score = 0;

        // Check title/description
        const description = sharedPackage.metadata.description?.toLowerCase() || '';
        if (description.includes(queryLower)) {
            score += 0.5;
        }

        // Check tags
        for (const tag of sharedPackage.metadata.tags) {
            if (tag.toLowerCase().includes(queryLower)) {
                score += 0.3;
            }
        }

        // Check context content (simplified)
        const contextString = JSON.stringify(sharedPackage.common_context).toLowerCase();
        if (contextString.includes(queryLower)) {
            score += 0.4;
        }

        return Math.min(score, 1.0);
    }

    private analyzeSharingPatterns(sharedContexts: SharedContextPackage[]): SharingPattern[] {
        // Analyze patterns in context sharing
        const patterns: SharingPattern[] = [];
        
        // Most active sharers
        const sharerCounts = new Map<string, number>();
        for (const pkg of sharedContexts) {
            sharerCounts.set(pkg.shared_by, (sharerCounts.get(pkg.shared_by) || 0) + 1);
        }
        
        patterns.push({
            type: 'most_active_sharers',
            data: Object.fromEntries(sharerCounts),
            insight: 'Users who share context most frequently'
        });

        return patterns;
    }

    private calculateCollaborationMetrics(
        sharedContexts: SharedContextPackage[],
        members: TeamMember[]
    ): CollaborationMetrics {
        return {
            collaboration_score: this.calculateCollaborationScore(sharedContexts, members),
            knowledge_sharing_index: this.calculateKnowledgeSharingIndex(sharedContexts),
            team_engagement: this.calculateTeamEngagement(sharedContexts, members)
        };
    }

    private calculateCollaborationScore(sharedContexts: SharedContextPackage[], members: TeamMember[]): number {
        // Calculate based on sharing frequency and participation
        const totalPossibleShares = members.length * (members.length - 1);
        const actualShares = sharedContexts.length;
        return Math.min(actualShares / totalPossibleShares, 1.0);
    }

    private calculateKnowledgeSharingIndex(sharedContexts: SharedContextPackage[]): number {
        // Calculate based on access patterns and context quality
        const totalAccesses = sharedContexts.reduce((sum, pkg) => sum + pkg.metadata.access_count, 0);
        const avgAccessesPerContext = sharedContexts.length > 0 ? totalAccesses / sharedContexts.length : 0;
        return Math.min(avgAccessesPerContext / 10, 1.0); // Normalize to 0-1
    }

    private calculateTeamEngagement(sharedContexts: SharedContextPackage[], members: TeamMember[]): number {
        // Calculate based on how many team members are actively participating
        const activeMembers = new Set<string>();
        for (const pkg of sharedContexts) {
            activeMembers.add(pkg.shared_by);
            // Would also count members who access/comment on shared contexts
        }
        return activeMembers.size / members.length;
    }

    private countActiveCollaborators(sharedContexts: SharedContextPackage[]): number {
        const activeUsers = new Set<string>();
        for (const pkg of sharedContexts) {
            activeUsers.add(pkg.shared_by);
            // Would also count users who have accessed contexts recently
        }
        return activeUsers.size;
    }

    // Role-specific filtering methods
    private filterForDeveloper(context: any): any {
        return {
            functions: context.functions,
            classes: context.classes,
            imports: context.imports,
            code_examples: this.extractCodeExamples({ semantic_context: context } as any)
        };
    }

    private filterForArchitect(context: any): any {
        return {
            architectural_overview: this.extractProjectOverview({ semantic_context: context } as any),
            design_patterns: this.extractCommonPatterns({ semantic_context: context } as any),
            system_dependencies: context.imports
        };
    }

    private filterForTester(context: any): any {
        return {
            testable_functions: context.functions,
            edge_cases: this.extractEdgeCases({ semantic_context: context } as any),
            quality_metrics: this.extractQualityMetrics({ semantic_context: context } as any)
        };
    }

    private filterForManager(context: any): any {
        return {
            high_level_overview: this.extractProjectOverview({ semantic_context: context } as any),
            impact_analysis: this.extractProjectImpact({ semantic_context: context } as any)
        };
    }

    // Context extraction methods (simplified implementations)
    private extractProjectOverview(checkpoint: AIContextCheckpoint): any {
        return { overview: 'Project overview based on semantic context' };
    }

    private extractKeyComponents(checkpoint: AIContextCheckpoint): any[] {
        return [];
    }

    private extractCommonPatterns(checkpoint: AIContextCheckpoint): any[] {
        return checkpoint.intent_analysis?.design_patterns_used || [];
    }

    private extractImplementationDetails(checkpoint: AIContextCheckpoint): any {
        return { details: 'Implementation details' };
    }

    private extractCodeExamples(checkpoint: AIContextCheckpoint): any[] {
        return [];
    }

    private extractDebuggingInfo(checkpoint: AIContextCheckpoint): any {
        return { debugging: 'Debugging information' };
    }

    private extractSystemDesign(checkpoint: AIContextCheckpoint): any {
        return { design: 'System design information' };
    }

    private extractScalabilityInfo(checkpoint: AIContextCheckpoint): any {
        return { scalability: 'Scalability considerations' };
    }

    private extractTestScenarios(checkpoint: AIContextCheckpoint): any[] {
        return [];
    }

    private extractEdgeCases(checkpoint: AIContextCheckpoint): any[] {
        return [];
    }

    private extractQualityMetrics(checkpoint: AIContextCheckpoint): any {
        return { metrics: 'Quality metrics' };
    }

    private extractProjectImpact(checkpoint: AIContextCheckpoint): any {
        return { impact: 'Project impact analysis' };
    }

    private extractTimelineInfo(checkpoint: AIContextCheckpoint): any {
        return { timeline: 'Timeline implications' };
    }

    private extractResourceInfo(checkpoint: AIContextCheckpoint): any {
        return { resources: 'Resource requirements' };
    }

    private async getUserCustomizations(userId: string): Promise<any> {
        return { preferences: 'User customizations' };
    }
}

// Supporting classes

class TeamManager {
    private config: CollaborativeConfig;

    constructor(config: CollaborativeConfig) {
        this.config = config;
    }

    async getTeam(teamId: string): Promise<Team | null> {
        // Mock implementation
        return {
            id: teamId,
            name: `Team ${teamId}`,
            members: [
                { id: 'user1', role: 'developer', name: 'Developer One' },
                { id: 'user2', role: 'architect', name: 'Architect One' },
                { id: 'user3', role: 'tester', name: 'Tester One' }
            ]
        };
    }

    async getUserRole(userId: string, teamId: string): Promise<string> {
        const team = await this.getTeam(teamId);
        const member = team?.members.find(m => m.id === userId);
        return member?.role || 'developer';
    }
}

class ContextSyncService {
    private config: CollaborativeConfig;

    constructor(config: CollaborativeConfig) {
        this.config = config;
    }

    async syncToTeam(sharedPackage: SharedContextPackage, members: TeamMember[]): Promise<void> {
        // Implementation for real-time sync
        console.log(`Syncing context ${sharedPackage.id} to ${members.length} team members`);
    }

    async syncUpdates(sharedPackage: SharedContextPackage, updates: ContextUpdate[], members: TeamMember[]): Promise<void> {
        // Implementation for syncing updates
        console.log(`Syncing ${updates.length} updates for context ${sharedPackage.id}`);
    }
}

class PermissionManager {
    async checkSharePermission(userId: string, teamId: string): Promise<void> {
        // Check if user can share context with team
    }

    async checkReadPermission(userId: string, sharedPackage: SharedContextPackage): Promise<boolean> {
        return sharedPackage.permissions.read.includes(userId);
    }

    async checkWritePermission(userId: string, sharedPackage: SharedContextPackage): Promise<boolean> {
        return sharedPackage.permissions.write.includes(userId);
    }

    async checkAdminPermission(userId: string, teamId: string): Promise<boolean> {
        return true; // Mock implementation
    }
}

// Interfaces

export interface CollaborativeConfig {
    maxSharedContexts?: number;
    syncIntervalMs?: number;
    maxTeamSize?: number;
    retentionDays?: number;
    enableRealTimeSync?: boolean;
    encryptSharedData?: boolean;
}

export interface Team {
    id: string;
    name: string;
    members: TeamMember[];
}

export interface TeamMember {
    id: string;
    role: string;
    name: string;
}

export interface ShareOptions {
    description?: string;
    tags?: string[];
    allowEdit?: boolean;
    restrictedAccess?: boolean;
    allowedUsers?: string[];
}

export interface SharedContextPackage {
    id: string;
    checkpoint_id: string;
    team_id: string;
    shared_by: string;
    common_context: any;
    personalized_views: PersonalizedView[];
    role_specific_context: { [role: string]: any };
    metadata: SharedContextMetadata;
    permissions: ContextPermissions;
    encrypted?: boolean;
}

interface PersonalizedView {
    member_id: string;
    role: string;
    context: any;
    access_level: AccessLevel;
    customizations: any;
}

interface SharedContextMetadata {
    created_at: Date;
    expires_at: Date;
    access_count: number;
    last_accessed: Date;
    tags: string[];
    description?: string;
}

interface ContextPermissions {
    read: string[];
    write: string[];
    admin: string[];
}

type AccessLevel = 'none' | 'limited' | 'standard' | 'full';

export interface PersonalizedSharedContext {
    id: string;
    common_context: any;
    personalized_context: any;
    role_context: any;
    metadata: SharedContextMetadata;
    shared_by: string;
}

export interface ContextUpdate {
    type: 'annotation' | 'highlight' | 'comment';
    target: string;
    content: any;
    position?: any;
}

export interface SearchFilters {
    tags?: string[];
    sharedBy?: string;
    dateRange?: {
        start?: Date;
        end?: Date;
    };
}

export interface SharedContextSearchResult {
    id: string;
    title: string;
    description: string;
    shared_by: string;
    created_at: Date;
    relevance_score: number;
    tags: string[];
    access_count: number;
}

export interface CollaborativeAnalytics {
    team_id: string;
    total_shared_contexts: number;
    total_accesses: number;
    average_accesses_per_context: number;
    active_collaborators: number;
    sharing_patterns: SharingPattern[];
    collaboration_metrics: CollaborationMetrics;
    generated_at: Date;
}

interface SharingPattern {
    type: string;
    data: any;
    insight: string;
}

interface CollaborationMetrics {
    collaboration_score: number;
    knowledge_sharing_index: number;
    team_engagement: number;
}
