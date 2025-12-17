/**
 * Real-Time Collaboration System - Live context sharing and synchronization
 * 
 * This system enables real-time collaborative AI context sharing, allowing
 * team members to work together on code understanding in real-time.
 */

import { SharedContextPackage, TeamMember } from '../collaborative/CollaborativeContextManager';
import { CompleteAIContext, QueryIntent } from '../AIContextBuilder';

export class RealTimeCollaboration {
    private webSocketManager: WebSocketManager;
    private sessionManager: CollaborationSessionManager;
    private conflictResolver: ConflictResolver;
    private presenceManager: PresenceManager;
    private config: RealTimeConfig;

    constructor(config: RealTimeConfig = {}) {
        this.config = {
            maxConcurrentSessions: config.maxConcurrentSessions || 10,
            sessionTimeoutMs: config.sessionTimeoutMs || 300000, // 5 minutes
            maxParticipants: config.maxParticipants || 20,
            enableConflictResolution: config.enableConflictResolution !== false,
            enablePresenceTracking: config.enablePresenceTracking !== false,
            broadcastIntervalMs: config.broadcastIntervalMs || 1000
        };

        this.webSocketManager = new WebSocketManager(this.config);
        this.sessionManager = new CollaborationSessionManager(this.config);
        this.conflictResolver = new ConflictResolver();
        this.presenceManager = new PresenceManager(this.config);
    }

    /**
     * Create a new real-time collaboration session
     */
    async createCollaborationSession(
        initiatorId: string,
        teamId: string,
        contextPackage: SharedContextPackage,
        sessionOptions: SessionOptions = {}
    ): Promise<CollaborationSession> {
        try {
            // Create the session
            const session = await this.sessionManager.createSession({
                initiator_id: initiatorId,
                team_id: teamId,
                context_package: contextPackage,
                max_participants: sessionOptions.maxParticipants ?? this.config.maxParticipants ?? 10,
                allow_anonymous: sessionOptions.allowAnonymous || false,
                session_type: sessionOptions.sessionType || 'general',
                permissions: sessionOptions.permissions || {
                    can_edit: [initiatorId],
                    can_view: [],
                    can_comment: []
                }
            });

            // Set up WebSocket handlers for this session
            await this.webSocketManager.setupSessionHandlers(session.id);

            // Initialize presence tracking
            await this.presenceManager.initializeSession(session.id);

            // Notify team members about the new session
            await this.notifyTeamMembers(session, 'session_created');

            return session;

        } catch (error) {
            console.error('Failed to create collaboration session:', error);
            throw new Error(`Session creation failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    /**
     * Join an existing collaboration session
     */
    async joinSession(
        sessionId: string,
        userId: string,
        userInfo: ParticipantInfo
    ): Promise<SessionJoinResult> {
        try {
            const session = await this.sessionManager.getSession(sessionId);
            if (!session) {
                return {
                    success: false,
                    error: 'Session not found'
                };
            }

            // Check permissions
            if (!await this.canJoinSession(session, userId)) {
                return {
                    success: false,
                    error: 'Permission denied'
                };
            }

            // Add participant to session
            const participant = await this.sessionManager.addParticipant(sessionId, userId, userInfo);

            // Set up WebSocket connection for user
            const connection = await this.webSocketManager.connectUser(sessionId, userId);

            // Update presence
            await this.presenceManager.userJoined(sessionId, userId, userInfo);

            // Send current session state to new participant
            await this.sendSessionState(sessionId, userId);

            // Notify other participants
            await this.broadcastToSession(sessionId, {
                type: 'participant_joined',
                participant,
                timestamp: new Date()
            }, userId);

            return {
                success: true,
                session,
                participant,
                connection_id: connection.id
            };

        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }

    /**
     * Leave a collaboration session
     */
    async leaveSession(sessionId: string, userId: string): Promise<void> {
        try {
            // Remove participant
            await this.sessionManager.removeParticipant(sessionId, userId);

            // Disconnect WebSocket
            await this.webSocketManager.disconnectUser(sessionId, userId);

            // Update presence
            await this.presenceManager.userLeft(sessionId, userId);

            // Notify other participants
            await this.broadcastToSession(sessionId, {
                type: 'participant_left',
                user_id: userId,
                timestamp: new Date()
            }, userId);

            // Clean up session if empty
            const session = await this.sessionManager.getSession(sessionId);
            if (session && session.participants.length === 0) {
                await this.endSession(sessionId);
            }

        } catch (error) {
            console.error('Failed to leave session:', error);
        }
    }

    /**
     * Share live context updates in real-time
     */
    async shareLiveContextUpdate(
        sessionId: string,
        userId: string,
        update: LiveContextUpdate
    ): Promise<void> {
        try {
            const session = await this.sessionManager.getSession(sessionId);
            if (!session) {
                throw new Error('Session not found');
            }

            // Check permissions
            if (!await this.canEditContext(session, userId)) {
                throw new Error('Edit permission denied');
            }

            // Apply conflict resolution if needed
            const resolvedUpdate = await this.conflictResolver.resolveUpdate(session, update);

            // Apply update to session context
            await this.sessionManager.applyContextUpdate(sessionId, resolvedUpdate);

            // Broadcast update to all participants
            await this.broadcastToSession(sessionId, {
                type: 'context_update',
                update: resolvedUpdate,
                updated_by: userId,
                timestamp: new Date()
            });

        } catch (error) {
            console.error('Failed to share live context update:', error);
            throw error;
        }
    }

    /**
     * Broadcast cursor/selection changes for awareness
     */
    async broadcastCursorUpdate(
        sessionId: string,
        userId: string,
        cursorUpdate: CursorUpdate
    ): Promise<void> {
        // Update presence with cursor information
        await this.presenceManager.updateCursor(sessionId, userId, cursorUpdate);

        // Broadcast to other participants (excluding sender)
        await this.broadcastToSession(sessionId, {
            type: 'cursor_update',
            user_id: userId,
            cursor: cursorUpdate,
            timestamp: new Date()
        }, userId);
    }

    /**
     * Handle real-time comments and annotations
     */
    async addLiveComment(
        sessionId: string,
        userId: string,
        comment: LiveComment
    ): Promise<void> {
        try {
            // Add comment to session
            const commentId = await this.sessionManager.addComment(sessionId, userId, comment);

            // Broadcast comment to all participants
            await this.broadcastToSession(sessionId, {
                type: 'comment_added',
                comment: {
                    ...comment,
                    id: commentId,
                    author_id: userId,
                    timestamp: new Date()
                }
            });

        } catch (error) {
            console.error('Failed to add live comment:', error);
            throw error;
        }
    }

    /**
     * Get real-time session analytics
     */
    async getSessionAnalytics(sessionId: string): Promise<SessionAnalytics> {
        const session = await this.sessionManager.getSession(sessionId);
        if (!session) {
            throw new Error('Session not found');
        }

        const presence = await this.presenceManager.getSessionPresence(sessionId);
        const activity = await this.sessionManager.getSessionActivity(sessionId);

        return {
            session_id: sessionId,
            duration_ms: Date.now() - session.created_at.getTime(),
            total_participants: session.participants.length,
            active_participants: presence.active_users.length,
            total_updates: activity.total_updates,
            total_comments: activity.total_comments,
            engagement_score: this.calculateEngagementScore(session, presence, activity),
            collaboration_effectiveness: this.calculateCollaborationEffectiveness(activity),
            generated_at: new Date()
        };
    }

    /**
     * End a collaboration session
     */
    async endSession(sessionId: string, endedBy?: string): Promise<void> {
        try {
            const session = await this.sessionManager.getSession(sessionId);
            if (!session) {
                return;
            }

            // Notify all participants
            await this.broadcastToSession(sessionId, {
                type: 'session_ended',
                ended_by: endedBy,
                timestamp: new Date()
            });

            // Disconnect all users
            for (const participant of session.participants) {
                await this.webSocketManager.disconnectUser(sessionId, participant.user_id);
            }

            // Clean up resources
            await this.sessionManager.endSession(sessionId);
            await this.presenceManager.cleanupSession(sessionId);
            await this.webSocketManager.cleanupSession(sessionId);

        } catch (error) {
            console.error('Failed to end session:', error);
        }
    }

    // Helper methods

    private async canJoinSession(session: CollaborationSession, userId: string): Promise<boolean> {
        // Check if user is team member
        if (session.team_id) {
            // Would check team membership in real implementation
        }

        // Check participant limit
        if (session.participants.length >= session.max_participants) {
            return false;
        }

        // Check permissions
        if (session.permissions.can_view.length > 0 && !session.permissions.can_view.includes(userId)) {
            return false;
        }

        return true;
    }

    private async canEditContext(session: CollaborationSession, userId: string): Promise<boolean> {
        return session.permissions.can_edit.includes(userId) || session.initiator_id === userId;
    }

    private async sendSessionState(sessionId: string, userId: string): Promise<void> {
        const session = await this.sessionManager.getSession(sessionId);
        const presence = await this.presenceManager.getSessionPresence(sessionId);

        await this.webSocketManager.sendToUser(sessionId, userId, {
            type: 'session_state',
            session,
            presence,
            timestamp: new Date()
        });
    }

    private async broadcastToSession(
        sessionId: string,
        message: any,
        excludeUserId?: string
    ): Promise<void> {
        await this.webSocketManager.broadcastToSession(sessionId, message, excludeUserId);
    }

    private async notifyTeamMembers(session: CollaborationSession, eventType: string): Promise<void> {
        // Implementation would send notifications to team members
        console.log(`Notifying team ${session.team_id} about ${eventType} for session ${session.id}`);
    }

    private calculateEngagementScore(
        session: CollaborationSession,
        presence: SessionPresence,
        activity: SessionActivity
    ): number {
        let score = 0;

        // Participation rate
        const participationRate = presence.active_users.length / session.participants.length;
        score += participationRate * 0.4;

        // Activity level
        const activityRate = activity.total_updates / Math.max(1, session.participants.length);
        score += Math.min(1, activityRate / 10) * 0.3;

        // Comment engagement
        const commentRate = activity.total_comments / Math.max(1, session.participants.length);
        score += Math.min(1, commentRate / 5) * 0.3;

        return score;
    }

    private calculateCollaborationEffectiveness(activity: SessionActivity): number {
        // Simple effectiveness calculation based on activity patterns
        const updatesPerParticipant = activity.total_updates / Math.max(1, activity.unique_contributors);
        const commentsPerUpdate = activity.total_comments / Math.max(1, activity.total_updates);

        return Math.min(1, (updatesPerParticipant * 0.6 + commentsPerUpdate * 0.4) / 10);
    }
}

// Supporting classes

class WebSocketManager {
    private connections: Map<string, Map<string, WebSocketConnection>> = new Map();
    private config: RealTimeConfig;

    constructor(config: RealTimeConfig) {
        this.config = config;
    }

    async setupSessionHandlers(sessionId: string): Promise<void> {
        if (!this.connections.has(sessionId)) {
            this.connections.set(sessionId, new Map());
        }
    }

    async connectUser(sessionId: string, userId: string): Promise<WebSocketConnection> {
        const connection: WebSocketConnection = {
            id: `conn_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            user_id: userId,
            session_id: sessionId,
            connected_at: new Date(),
            last_activity: new Date()
        };

        const sessionConnections = this.connections.get(sessionId);
        if (sessionConnections) {
            sessionConnections.set(userId, connection);
        }

        return connection;
    }

    async disconnectUser(sessionId: string, userId: string): Promise<void> {
        const sessionConnections = this.connections.get(sessionId);
        if (sessionConnections) {
            sessionConnections.delete(userId);
        }
    }

    async sendToUser(sessionId: string, userId: string, message: any): Promise<void> {
        const sessionConnections = this.connections.get(sessionId);
        const connection = sessionConnections?.get(userId);
        
        if (connection) {
            // In real implementation, would send via WebSocket
            console.log(`Sending to user ${userId}:`, message.type);
        }
    }

    async broadcastToSession(sessionId: string, message: any, excludeUserId?: string): Promise<void> {
        const sessionConnections = this.connections.get(sessionId);
        if (sessionConnections) {
            for (const [userId, connection] of sessionConnections) {
                if (userId !== excludeUserId) {
                    await this.sendToUser(sessionId, userId, message);
                }
            }
        }
    }

    async cleanupSession(sessionId: string): Promise<void> {
        this.connections.delete(sessionId);
    }
}

class CollaborationSessionManager {
    private sessions: Map<string, CollaborationSession> = new Map();
    private config: RealTimeConfig;

    constructor(config: RealTimeConfig) {
        this.config = config;
    }

    async createSession(options: CreateSessionOptions): Promise<CollaborationSession> {
        const session: CollaborationSession = {
            id: this.generateSessionId(),
            initiator_id: options.initiator_id,
            team_id: options.team_id,
            context_package: options.context_package,
            participants: [],
            max_participants: options.max_participants,
            created_at: new Date(),
            last_activity: new Date(),
            session_type: options.session_type,
            permissions: options.permissions,
            comments: [],
            context_updates: []
        };

        this.sessions.set(session.id, session);
        return session;
    }

    async getSession(sessionId: string): Promise<CollaborationSession | null> {
        return this.sessions.get(sessionId) || null;
    }

    async addParticipant(
        sessionId: string,
        userId: string,
        userInfo: ParticipantInfo
    ): Promise<SessionParticipant> {
        const session = this.sessions.get(sessionId);
        if (!session) {
            throw new Error('Session not found');
        }

        const participant: SessionParticipant = {
            user_id: userId,
            name: userInfo.name,
            role: userInfo.role,
            joined_at: new Date(),
            is_active: true,
            permissions: this.getParticipantPermissions(session, userId)
        };

        session.participants.push(participant);
        session.last_activity = new Date();

        return participant;
    }

    async removeParticipant(sessionId: string, userId: string): Promise<void> {
        const session = this.sessions.get(sessionId);
        if (session) {
            session.participants = session.participants.filter(p => p.user_id !== userId);
            session.last_activity = new Date();
        }
    }

    async applyContextUpdate(sessionId: string, update: LiveContextUpdate): Promise<void> {
        const session = this.sessions.get(sessionId);
        if (session) {
            session.context_updates.push({
                ...update,
                timestamp: new Date()
            });
            session.last_activity = new Date();
        }
    }

    async addComment(sessionId: string, userId: string, comment: LiveComment): Promise<string> {
        const session = this.sessions.get(sessionId);
        if (!session) {
            throw new Error('Session not found');
        }

        const commentId = `comment_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        
        session.comments.push({
            id: commentId,
            author_id: userId,
            content: comment.content,
            target: comment.target,
            position: comment.position,
            timestamp: new Date()
        });

        session.last_activity = new Date();
        return commentId;
    }

    async getSessionActivity(sessionId: string): Promise<SessionActivity> {
        const session = this.sessions.get(sessionId);
        if (!session) {
            throw new Error('Session not found');
        }

        const uniqueContributors = new Set([
            ...session.context_updates.map(u => u.updated_by),
            ...session.comments.map(c => c.author_id)
        ]).size;

        return {
            total_updates: session.context_updates.length,
            total_comments: session.comments.length,
            unique_contributors: uniqueContributors
        };
    }

    async endSession(sessionId: string): Promise<void> {
        this.sessions.delete(sessionId);
    }

    private generateSessionId(): string {
        return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    private getParticipantPermissions(session: CollaborationSession, userId: string): ParticipantPermissions {
        return {
            can_edit: session.permissions.can_edit.includes(userId) || session.initiator_id === userId,
            can_comment: session.permissions.can_comment.includes(userId) || 
                        session.permissions.can_comment.length === 0,
            can_view: true
        };
    }
}

class ConflictResolver {
    async resolveUpdate(session: CollaborationSession, update: LiveContextUpdate): Promise<LiveContextUpdate> {
        // Simple conflict resolution - in practice would be more sophisticated
        
        // Check for concurrent updates to the same target
        const recentUpdates = session.context_updates.filter(u => 
            u.target === update.target && 
            Date.now() - u.timestamp.getTime() < 5000 // Within last 5 seconds
        );

        if (recentUpdates.length > 0) {
            // Apply operational transformation or merge strategy
            update.conflict_resolution = {
                strategy: 'merge',
                base_version: recentUpdates[recentUpdates.length - 1].version || 0,
                resolved_at: new Date()
            };
        }

        return update;
    }
}

class PresenceManager {
    private sessionPresence: Map<string, SessionPresence> = new Map();
    private config: RealTimeConfig;

    constructor(config: RealTimeConfig) {
        this.config = config;
    }

    async initializeSession(sessionId: string): Promise<void> {
        this.sessionPresence.set(sessionId, {
            session_id: sessionId,
            active_users: [],
            user_cursors: new Map(),
            last_updated: new Date()
        });
    }

    async userJoined(sessionId: string, userId: string, userInfo: ParticipantInfo): Promise<void> {
        const presence = this.sessionPresence.get(sessionId);
        if (presence) {
            presence.active_users.push({
                user_id: userId,
                name: userInfo.name,
                status: 'active',
                last_seen: new Date()
            });
            presence.last_updated = new Date();
        }
    }

    async userLeft(sessionId: string, userId: string): Promise<void> {
        const presence = this.sessionPresence.get(sessionId);
        if (presence) {
            presence.active_users = presence.active_users.filter(u => u.user_id !== userId);
            presence.user_cursors.delete(userId);
            presence.last_updated = new Date();
        }
    }

    async updateCursor(sessionId: string, userId: string, cursor: CursorUpdate): Promise<void> {
        const presence = this.sessionPresence.get(sessionId);
        if (presence) {
            presence.user_cursors.set(userId, {
                ...cursor,
                updated_at: new Date()
            });
            presence.last_updated = new Date();
        }
    }

    async getSessionPresence(sessionId: string): Promise<SessionPresence> {
        return this.sessionPresence.get(sessionId) || {
            session_id: sessionId,
            active_users: [],
            user_cursors: new Map(),
            last_updated: new Date()
        };
    }

    async cleanupSession(sessionId: string): Promise<void> {
        this.sessionPresence.delete(sessionId);
    }
}

// Interfaces and types

export interface RealTimeConfig {
    maxConcurrentSessions?: number;
    sessionTimeoutMs?: number;
    maxParticipants?: number;
    enableConflictResolution?: boolean;
    enablePresenceTracking?: boolean;
    broadcastIntervalMs?: number;
}

export interface SessionOptions {
    maxParticipants?: number;
    allowAnonymous?: boolean;
    sessionType?: 'general' | 'debugging' | 'architecture' | 'review';
    permissions?: SessionPermissions;
}

export interface CollaborationSession {
    id: string;
    initiator_id: string;
    team_id: string;
    context_package: SharedContextPackage;
    participants: SessionParticipant[];
    max_participants: number;
    created_at: Date;
    last_activity: Date;
    session_type: string;
    permissions: SessionPermissions;
    comments: SessionComment[];
    context_updates: TimestampedContextUpdate[];
}

interface SessionParticipant {
    user_id: string;
    name: string;
    role: string;
    joined_at: Date;
    is_active: boolean;
    permissions: ParticipantPermissions;
}

interface SessionPermissions {
    can_edit: string[];
    can_view: string[];
    can_comment: string[];
}

interface ParticipantPermissions {
    can_edit: boolean;
    can_comment: boolean;
    can_view: boolean;
}

export interface ParticipantInfo {
    name: string;
    role: string;
    avatar?: string;
}

export interface SessionJoinResult {
    success: boolean;
    session?: CollaborationSession;
    participant?: SessionParticipant;
    connection_id?: string;
    error?: string;
}

export interface LiveContextUpdate {
    id?: string;
    target: string; // What part of context is being updated
    operation: 'add' | 'modify' | 'delete' | 'highlight' | 'annotate';
    content: any;
    position?: ContextPosition;
    updated_by?: string;
    version?: number;
    conflict_resolution?: ConflictResolution;
}

interface TimestampedContextUpdate extends LiveContextUpdate {
    timestamp: Date;
}

export interface CursorUpdate {
    position: ContextPosition;
    selection?: ContextRange;
    color?: string;
}

export interface LiveComment {
    content: string;
    target: string;
    position: ContextPosition;
    thread_id?: string;
}

interface SessionComment extends LiveComment {
    id: string;
    author_id: string;
    timestamp: Date;
}

interface ContextPosition {
    file?: string;
    line?: number;
    column?: number;
    element_id?: string;
}

interface ContextRange {
    start: ContextPosition;
    end: ContextPosition;
}

interface ConflictResolution {
    strategy: 'merge' | 'overwrite' | 'manual';
    base_version: number;
    resolved_at: Date;
}

interface WebSocketConnection {
    id: string;
    user_id: string;
    session_id: string;
    connected_at: Date;
    last_activity: Date;
}

interface CreateSessionOptions {
    initiator_id: string;
    team_id: string;
    context_package: SharedContextPackage;
    max_participants: number;
    allow_anonymous: boolean;
    session_type: string;
    permissions: SessionPermissions;
}

interface SessionPresence {
    session_id: string;
    active_users: ActiveUser[];
    user_cursors: Map<string, CursorInfo>;
    last_updated: Date;
}

interface ActiveUser {
    user_id: string;
    name: string;
    status: 'active' | 'idle' | 'away';
    last_seen: Date;
}

interface CursorInfo extends CursorUpdate {
    updated_at: Date;
}

interface SessionActivity {
    total_updates: number;
    total_comments: number;
    unique_contributors: number;
}

export interface SessionAnalytics {
    session_id: string;
    duration_ms: number;
    total_participants: number;
    active_participants: number;
    total_updates: number;
    total_comments: number;
    engagement_score: number;
    collaboration_effectiveness: number;
    generated_at: Date;
}
