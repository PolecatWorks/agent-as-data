{{/*
Expand the name of the chart.
*/}}
{{- define "agent-as-data.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

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
