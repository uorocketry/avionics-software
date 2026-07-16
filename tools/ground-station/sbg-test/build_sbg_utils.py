import os
import subprocess
import shutil
import sys
import platform
import glob


def clone_sbg_ecom_repo():
    """Clone the sbgECom repository if it doesn't exist."""
    repo_url = "https://github.com/SBG-Systems/sbgECom"
    repo_dir = os.path.join(os.path.dirname(__file__), "sbgECom")
    if not os.path.exists(repo_dir):
        print(f"Cloning {repo_url} into {repo_dir}...")
        subprocess.check_call(["git", "clone", repo_url, repo_dir])
    else:
        print(f"Repository already cloned at {repo_dir}.")
    return repo_dir


def check_dependencies():
    """Check if required build dependencies are available."""
    required_tools = ["cmake", "git"]

    # On POSIX, ensure compilers exist (Windows handled by generator toolchains)
    if os.name == "posix":
        required_tools.extend(["gcc", "g++"])

    missing_tools = []
    for tool in required_tools:
        try:
            subprocess.run([tool, "--version"], capture_output=True, check=True)
            print(f"✓ {tool} found")
        except (subprocess.CalledProcessError, FileNotFoundError):
            missing_tools.append(tool)
            print(f"✗ {tool} not found")

    if missing_tools:
        print(f"\nMissing required tools: {', '.join(missing_tools)}")
        print("Please install the missing tools before running this script.")
        sys.exit(1)


def configure_project(repo_dir: str, build_dir: str) -> None:
    """Run CMake configuration step."""
    print("Configuring build with CMake...")
    cmake_cmd = [
        "cmake",
        "..",
        "-DBUILD_TOOLS=ON",
        "-DBUILD_EXAMPLES=OFF",
        "-DCMAKE_BUILD_TYPE=Release",
        "-DBUILD_SHARED_LIBS=OFF",  # Build static libraries for sbgECom
        "-DCMAKE_POLICY_VERSION_MINIMUM=3.5",
    ]
    subprocess.check_call(cmake_cmd, cwd=build_dir)


def build_project(build_dir: str) -> None:
    """Build using CMake's cross-platform build command."""
    print("Building binaries...")
    # Use cmake --build for portability (handles make/ninja/msbuild)
    # Use Release config for multi-config generators (e.g., Visual Studio)
    build_cmd = ["cmake", "--build", ".", "--config", "Release", "--parallel"]
    subprocess.check_call(build_cmd, cwd=build_dir)


def find_binary(build_dir: str, base_name: str) -> str | None:
    """Locate a built binary, accounting for generator layouts and platforms."""
    exe_suffix = ".exe" if platform.system() == "Windows" else ""
    candidates = [
        os.path.join(build_dir, f"{base_name}{exe_suffix}"),
        os.path.join(build_dir, "Release", f"{base_name}{exe_suffix}"),
    ]

    for path in candidates:
        if os.path.exists(path):
            return path

    # Fallback: search within build dir but avoid CMakeFiles artifacts
    for root, dirs, files in os.walk(build_dir):
        # Skip internal build directories
        if "CMakeFiles" in root:
            continue
        for fname in files:
            if fname == f"{base_name}{exe_suffix}":
                return os.path.join(root, fname)
    return None


def build_binaries(repo_dir: str) -> str:
    """Configure and build the sbgBasicLogger and sbgEComApi binaries."""
    build_dir = os.path.join(repo_dir, "build")

    # Create fresh build directory
    if os.path.exists(build_dir):
        print(f"Cleaning existing build directory: {build_dir}")
        shutil.rmtree(build_dir)
    os.makedirs(build_dir, exist_ok=True)

    configure_project(repo_dir, build_dir)
    build_project(build_dir)
    return build_dir


def copy_binaries(build_dir: str, target_dir: str) -> list[str]:
    """Copy the built binaries to the target directory."""
    print(f"Copying binaries to {target_dir}...")

    binary_names = ["sbgBasicLogger", "sbgEComApi"]
    copied_binaries: list[str] = []

    for name in binary_names:
        src_path = find_binary(build_dir, name)
        if src_path and os.path.exists(src_path):
            # Ensure executable bit on POSIX
            if os.name == "posix":
                os.chmod(src_path, 0o755)
            dst_basename = os.path.basename(src_path)
            dst_path = os.path.join(target_dir, dst_basename)
            shutil.copy2(src_path, dst_path)
            print(f"✓ Copied {dst_basename} to {dst_path}")
            copied_binaries.append(dst_path)
        else:
            print(f"✗ Binary not found: {name}")

    return copied_binaries


def verify_binaries(binaries: list[str]) -> None:
    """Verify that the binaries exist and are runnable."""
    print("\nVerifying binaries...")
    for binary_path in binaries:
        if not os.path.exists(binary_path):
            print(f"✗ {os.path.basename(binary_path)} not found")
            continue
        if os.name == "posix" and not os.access(binary_path, os.X_OK):
            print(f"✗ {os.path.basename(binary_path)} is not executable")
        else:
            print(f"✓ {os.path.basename(binary_path)} is ready")


def main():
    """Main function to orchestrate the build process."""
    print("=== SBG ECom Tools Build Automation ===\n")

    # Check dependencies
    print("1. Checking build dependencies...")
    check_dependencies()

    # Clone repository
    print("\n2. Setting up repository...")
    repo_dir = clone_sbg_ecom_repo()

    # Build binaries
    print("\n3. Building binaries...")
    build_dir = build_binaries(repo_dir)

    # Copy binaries to current directory
    print("\n4. Copying binaries...")
    target_dir = os.path.dirname(__file__)
    copied_binaries = copy_binaries(build_dir, target_dir)

    # Verify binaries
    print("\n5. Verifying binaries...")
    verify_binaries(copied_binaries)

    print(f"\n=== Build Complete ===")
    print(f"Binaries available in: {target_dir}")
    for binary in copied_binaries:
        print(f"  - {os.path.basename(binary)}")


if __name__ == "__main__":
    main()
