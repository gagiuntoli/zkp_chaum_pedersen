# Use an existing Rust image as the base
FROM rust:1.68

# Set the working directory
WORKDIR /zpk-app

# Copy the application files into the image
COPY . .

RUN apt update
RUN apt-get -y install cmake

# Build the application in release mode
RUN cargo build --release