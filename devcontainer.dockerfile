# Use a Debian-based Linux distribution as the base image
FROM runtime AS base

# Set the working directory
WORKDIR /workshop

# Set the default command to run when the container starts
CMD ["bash"]