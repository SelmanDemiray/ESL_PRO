#!/bin/bash

PROJECT_NAME="mylove"

echo "🧹 Cleaning up Docker containers and images for $PROJECT_NAME..."

# Stop and remove containers
docker-compose down --rmi local --volumes --remove-orphans

# Remove images built for this project (by project directory)
docker images --format "{{.Repository}}:{{.Tag}} {{.ID}}" | grep "$PROJECT_NAME" | awk '{print $2}' | xargs -r docker rmi

# Clean up Cargo.lock for fresh build
rm -f Cargo.lock

echo "✅ Cleanup complete."
