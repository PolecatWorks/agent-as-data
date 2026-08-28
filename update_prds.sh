for file in spec/prds/*.md; do
  sed -i 's/\/api\/v1/\/{{api_prefix}}\/v1/g' "$file"
done
