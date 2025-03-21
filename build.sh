#!/bin/bash

VERSION="$1"
DEFAULT_VERSION="0.0.1"
LINE="--------------------------------------------------------------------------------"

message() {
    printf "%s\n%s\n%s\n" "$LINE" "$1" "$LINE"
}

# Check if the version is provided
if [ -z "$VERSION" ]; then
    message "Version number missing, using default: $DEFAULT_VERSION."
    VERSION=$DEFAULT_VERSION
fi

# Turning down docker-compose
message "Turning down docker-compose..."
docker-compose down

# Build the project
message "Building the project..."
docker build -t ninoverse:$VERSION .

# Run the project
message "Running the project..."
docker-compose up -d

# Check the status
message "Checking the status..."
docker-compose ps

# Wait for 10 seconds
message "Waiting for 10 seconds to let kafka start..."
sleep 10

# Creating kafka topics
message "Creating kafka topics..."
docker exec ninoverse-broker-1 /opt/kafka/bin/kafka-topics.sh --create --topic ninoverse --bootstrap-server broker:9092 --partitions 1 --replication-factor 1

# Done
message "Done."

message "Memory occupied by docker:"
docker system df

message "PC memory:"
df -h