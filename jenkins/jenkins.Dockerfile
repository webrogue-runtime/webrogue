FROM  jenkins/jenkins:lts-jdk11
RUN jenkins-plugin-cli --plugins "blueocean configuration-as-code job-dsl cloudbees-folder antisamy-markup-formatter build-timeout credentials-binding timestamper ws-cleanup workflow-aggregator pipeline-stage-view git ssh-slaves ssh-agent"

ENV CASC_JENKINS_CONFIG=/jenkins_config.yaml

COPY keys/jenkins_rsa /jenkins_rsa
COPY keys/android_store_password /android_store_password
COPY keys/android_key_password /android_key_password

COPY jenkins_config.yaml /jenkins_config.yaml
