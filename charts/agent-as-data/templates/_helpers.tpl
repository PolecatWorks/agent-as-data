{{/*
Expand the name of the chart.
*/}}
{{- define "agent-as-data.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "agent-as-data.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}



{{/*
Create a default fully qualified app name.
*/}}
{{- define "agent-as-data.fullname" -}}
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

{{- define "agent-as-data.config" -}}
{{- $baseconfig := fromYaml (.Files.Get "configs/config.yaml") }}
{{- $newconfig := default dict .Values.config }}
{{- $postmerge := mergeOverwrite $baseconfig $newconfig }}
{{- tpl (toYaml $postmerge) . }}
{{- end -}}

{{- define "agent-as-data.volumes" -}}
{{- tpl (toYaml .Values.volumes) . }}
{{- end -}}

{{- define "agent-as-data.volumeMounts" -}}
{{- tpl (toYaml .Values.volumeMounts) . }}
{{- end -}}

{{- define "agent-as-data.env" -}}
{{- tpl (toYaml .Values.env) . }}
{{- end -}}


{{/*
Common labels
*/}}
{{- define "agent-as-data.labels" -}}
helm.sh/chart: {{ include "agent-as-data.chart" . }}
{{ include "agent-as-data.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "agent-as-data.selectorLabels" -}}
app.kubernetes.io/name: {{ include "agent-as-data.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}
