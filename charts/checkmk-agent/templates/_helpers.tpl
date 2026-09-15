{{/*
Expand the name of the chart.
*/}}
{{- define "checkmk-agent.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "checkmk-agent.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "checkmk-agent.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "checkmk-agent.pullSharedSecretName" -}}
{{- if .Values.pull.authentication.existingSecret.name -}}
{{- .Values.pull.authentication.existingSecret.name -}}
{{- else -}}
{{- printf "%s-pull-agent-secret" (include "checkmk-agent.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "checkmk-agent.pullSharedSecretKey" -}}
{{- if and
  .Values.pull.authentication.existingSecret.name
  .Values.pull.authentication.existingSecret.key
-}}
{{- .Values.pull.authentication.existingSecret.key -}}
{{- else -}}
secret
{{- end -}}
{{- end -}}

{{- define "checkmk-agent.intraClusterTLSSecretName" -}}
{{- if .Values.intraClusterCommunication.encryption.existingSecret -}}
{{- .Values.intraClusterCommunication.encryption.existingSecret -}}
{{- else -}}
{{- printf "%s-intra-cluster-ca" (include "checkmk-agent.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "checkmk-agent.pullTLSSecretName" -}}
{{- if .Values.pull.encryption.existingSecret -}}
{{- .Values.pull.encryption.existingSecret -}}
{{- else -}}
{{- printf "%s-pull-ca" (include "checkmk-agent.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/* Standard resource labels. Keep version labels out of selectors. */}}
{{- define "checkmk-agent.labels" -}}
helm.sh/chart: {{ include "checkmk-agent.chart" . }}
app.kubernetes.io/name: {{ include "checkmk-agent.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/* A digest pins the image even when a tag is also configured. */}}
{{- define "checkmk-agent.image" -}}
{{- if .image.digest -}}
{{- printf "%s@%s" .image.repository .image.digest -}}
{{- else -}}
{{- printf "%s:%s" .image.repository (.image.tag | default .root.Chart.AppVersion) -}}
{{- end -}}
{{- end -}}

{{/* Each component needs its own identity for ingestion authorization. */}}
{{- define "checkmk-agent.serviceAccountName" -}}
{{- default (printf "%s-%s" (include "checkmk-agent.fullname" .root) .component) .serviceAccount.name -}}
{{- end -}}

{{/* SCCs are cluster-scoped, so include a namespace hash in their names. */}}
{{- define "checkmk-agent.nodeScraperSCCName" -}}
{{- printf "%s-%s" (include "checkmk-agent.fullname" . | trunc 54 | trimSuffix "-") (.Release.Namespace | sha256sum | trunc 8) -}}
{{- end -}}
