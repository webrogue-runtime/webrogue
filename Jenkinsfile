pipeline {
    agent none

    stages {
        stage('Build') {
            parallel {
                stage('Build for Android') {
                    agent { label 'linux' }

                    environment {
                        ANDROID_SDK_ROOT='/opt/android-sdk/'
                        ORG_GRADLE_PROJECT_RELEASE_STORE_PASSWORD = credentials('android_store_password')
                        ORG_GRADLE_PROJECT_RELEASE_KEY_PASSWORD = credentials('android_key_password')
                    }

                    steps {
                        sh 'rm -rf ./platforms/Android/app/build/outputs/apk/release/app-release.apk'
                        sh 'cd platforms/Android && ./gradlew --no-daemon assembleRelease'
                        archiveArtifacts artifacts: 'platforms/Android/app/build/outputs/apk/release/app-release.apk'
                    }
                }

                stage('Build for Windows') {
                    agent { label 'linux' }

                    steps {
                        sh 'rm -rf windows_artifacts.zip artifacts/'

                        sh 'export PATH=/opt/msvc/bin/x64:\$PATH; wineserver -p || true; sh platforms/Windows/build_wine.sh'

                        sh 'mkdir -p artifacts'
                        sh 'cd platforms/Windows/build/ && cp -r webrogue.exe webrogue_core.dll webrogue_runtime_wasmedge.dll webrogue_sdl.exe SDL2.dll SDL2_ttf.dll mods ../../../artifacts'
                        sh 'cd artifacts && zip -r ../windows_artifacts.zip .'
                        archiveArtifacts artifacts: 'windows_artifacts.zip'
                    }
                }

                stage('Build for Web') {
                    agent { label 'linux' }

                    steps {
                        sh 'rm -rf web_artifacts.zip artifacts/'

                        sh 'bash -c \'. /home/jenkins/emsdk/emsdk_env.sh && emcmake cmake -Bplatforms/Web/homepage/build -Splatforms/Web/homepage -DCMAKE_BUILD_TYPE=Release\''
                        sh 'cmake --build platforms/Web/homepage/build/ --target pack_artifacts --parallel'

                        sh 'cd artifacts && zip -r ../web_artifacts.zip .'
                        archiveArtifacts artifacts: 'web_artifacts.zip'
                    }
                }

                stage('Build for Linux') {
                    agent { label 'linux' }

                    steps {
                        sh 'rm -rf linux_artifacts.zip artifacts/'

                        sh 'cmake -S platforms/Linux -B platforms/Linux/build -DCMAKE_BUILD_TYPE=Release'
                        sh 'cmake --build platforms/Linux/build --target pack_executable_to_artifacts --parallel'

                        sh 'cd artifacts && zip -r ../linux_artifacts.zip .'
                        archiveArtifacts artifacts: 'linux_artifacts.zip'
                    }
                }

                stage('Build for Dos') {
                    agent { label 'linux' }

                    steps {
                        sh 'rm -rf dos_artifacts.zip artifacts/'

                        sh 'cmake --toolchain=djgpp_toolchain.cmake -S platforms/DOS/ -B platforms/DOS/build -DCMAKE_BUILD_TYPE=Release'
                        sh 'cmake --build platforms/DOS/build --target pack_executable_to_artifacts --parallel'

                        sh 'cd artifacts && zip -r ../dos_artifacts.zip .'
                        archiveArtifacts artifacts: 'dos_artifacts.zip'
                    }
                }
            }
        }
    }
}
