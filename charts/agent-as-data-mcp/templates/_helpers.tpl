{{/*
Expand the name of the chart.
*/}}
{{- define "agent-as-data-mcp.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "agent-as-data-mcp.fullname" -}}
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

{{- define "agent-as-data-mcp.config" -}}
{{- $baseconfig := fromYaml (.Files.Get "configs/config.yaml") }}
{{- $newconfig := default dict .Values.config }}
{{- $postmerge := mergeOverwrite $baseconfig $newconfig }}
{{- tpl (toYaml $postmerge) . }}
{{- end -}}

{{- define "agent-as-data-mcp.volumes" -}}
{{- tpl (toYaml .Values.volumes) . }}
{{- end -}}

{{- define "agent-as-data-mcp.volumeMounts" -}}
{{- tpl (toYaml .Values.volumeMounts) . }}
{{- end -}}

{{- define "agent-as-data-mcp.env" -}}
{{- tpl (toYaml .Values.env) . }}
{{- end -}}
