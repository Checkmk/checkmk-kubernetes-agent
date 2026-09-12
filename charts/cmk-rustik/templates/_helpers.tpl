{{/*
Expand the name of the chart.
*/}}
{{- define "rustik.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "rustik.fullname" -}}
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
{{- define "rustik.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "rustik.pullSharedSecretName" -}}
{{- if .Values.pull.authentication.existingSecret.name -}}
{{- .Values.pull.authentication.existingSecret.name -}}
{{- else -}}
{{- printf "%s-pull-agent-secret" (include "rustik.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "rustik.pullSharedSecretKey" -}}
{{- if and
  .Values.pull.authentication.existingSecret.name
  .Values.pull.authentication.existingSecret.key
-}}
{{- .Values.pull.authentication.existingSecret.key -}}
{{- else -}}
secret
{{- end -}}
{{- end -}}

{{- define "rustik.intraClusterTLSSecretName" -}}
{{- if .Values.intraClusterCommunication.encryption.existingSecret -}}
{{- .Values.intraClusterCommunication.encryption.existingSecret -}}
{{- else -}}
{{- printf "%s-intra-cluster-ca" (include "rustik.fullname" .) -}}
{{- end -}}
{{- end -}}

{{- define "rustik.pullTLSSecretName" -}}
{{- if .Values.pull.encryption.existingSecret -}}
{{- .Values.pull.encryption.existingSecret -}}
{{- else -}}
{{- printf "%s-pull-ca" (include "rustik.fullname" .) -}}
{{- end -}}
{{- end -}}

{{/* Standard resource labels. Keep version labels out of selectors. */}}
{{- define "rustik.labels" -}}
helm.sh/chart: {{ include "rustik.chart" . }}
app.kubernetes.io/name: {{ include "rustik.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/* A digest pins the image even when a tag is also configured. */}}
{{- define "rustik.image" -}}
{{- if .image.digest -}}
{{- printf "%s@%s" .image.repository .image.digest -}}
{{- else -}}
{{- printf "%s:%s" .image.repository (.image.tag | default .root.Chart.AppVersion) -}}
{{- end -}}
{{- end -}}

{{/* Each component needs its own identity for ingestion authorization. */}}
{{- define "rustik.serviceAccountName" -}}
{{- default (printf "%s-%s" (include "rustik.fullname" .root) .component) .serviceAccount.name -}}
{{- end -}}
