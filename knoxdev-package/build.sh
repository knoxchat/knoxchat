echo "Installing and Building config-types..."
pushd config-types
npm install
npm run build

popd

echo "Installing and Building config-yaml..."
pushd config-yaml
npm install
npm run build

popd

echo "Installing and Building fetch..."
pushd fetch
npm install
npm run build

popd

echo "Installing and Building llm-info..."
pushd llm-info
npm install
npm run build

popd

echo "Installing and Building openai-adapters..."
pushd openai-adapters
npm install
npm run build

popd