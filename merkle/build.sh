#!/bin/bash

# Name of your Rust binary
BINARY_NAME="merkle"

# Build the binary with musl target
cargo build --release --target x86_64-unknown-linux-musl

# Create a directory for the bootstrap file
mkdir -p lambda

# Copy the compiled binary and rename it to 'bootstrap'
cp ./target/x86_64-unknown-linux-musl/release/$BINARY_NAME ./lambda/bootstrap

# Change permissions to allow execution
chmod +x ./lambda/bootstrap

# Zip the bootstrap file
cd lambda
zip lambda.zip bootstrap

# Move the zip file to the cdk folder
mv lambda.zip ../../cdk-deploy

# Clean up
cd ..
rm -rf lambda

echo "Lambda deployment package created: lambda.zip"