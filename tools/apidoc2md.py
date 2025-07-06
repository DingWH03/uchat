import json

# Load the JSON file
with open("doc/api/openapi.json", "r", encoding='utf-8') as file:
    openapi_data = json.load(file)

def parse_schema(schema, components):
    """递归解析 schema，返回字段和类型描述"""
    if not schema:
        return ""
    result = ""
    if '$ref' in schema:
        ref_path = schema['$ref']
        # 仅处理 #/components/schemas/xxx
        ref_name = ref_path.split("/")[-1]
        ref_schema = components.get('schemas', {}).get(ref_name, {})
        return parse_schema(ref_schema, components)
    if schema.get('type') == 'object':
        properties = schema.get('properties', {})
        for prop, prop_schema in properties.items():
            type_name = prop_schema.get('type', 'object')
            result += f"    - `{prop}`: {type_name}\n"
            # 递归解析嵌套对象
            if type_name == 'object' or '$ref' in prop_schema:
                nested = parse_schema(prop_schema, components)
                if nested:
                    result += nested
    elif schema.get('type') == 'array':
        item_type = schema.get('items', {}).get('type', 'object')
        result += f"    - `[item]`: {item_type}\n"
        nested = parse_schema(schema.get('items', {}), components)
        if nested:
            result += nested
    return result

# Function to convert JSON to Markdown
def convert_to_markdown(openapi_data):
    components = openapi_data.get('components', {})
    markdown = f"# {openapi_data.get('info', {}).get('title', 'API Documentation')}\n\n"
    markdown += f"## 版本: {openapi_data.get('info', {}).get('version', 'N/A')}\n\n"

    paths = openapi_data.get("paths", {})
    for path, methods in paths.items():
        for method, details in methods.items():
            markdown += f"### {path}\n\n"
            markdown += f"- **方法**: {method.upper()}\n"
            markdown += f"- **标签**: {', '.join(details.get('tags', []))}\n"
            markdown += f"- **摘要**: {details.get('summary', '')}\n"
            markdown += f"- **描述**: {details.get('description', '')}\n"

            parameters = details.get('parameters', [])
            if parameters:
                markdown += "- **参数**:\n"
                for param in parameters:
                    markdown += f"  - **{param.get('name')}** ({param.get('in')}, {param.get('schema', {}).get('type', 'N/A')}, "
                    markdown += "required): " if param.get('required') else "optional): "
                    markdown += f"{param.get('description', '')}\n"

            request_body = details.get('requestBody', {})
            if request_body:
                markdown += "- **请求体**:\n"
                for content_type, schema in request_body.get('content', {}).items():
                    markdown += f"  - **{content_type}**:\n"
                    ref = schema.get('schema', {}).get('$ref')
                    if ref:
                        ref_name = ref.split('/')[-1]
                        markdown += f"    - **字段**:\n"
                        ref_schema = components.get('schemas', {}).get(ref_name, {})
                        markdown += parse_schema(ref_schema, components)
                    else:
                        markdown += f"    - **字段**: {schema.get('schema', {})}\n"

            responses = details.get('responses', {})
            if responses:
                markdown += "- **响应**:\n"
                for status, response in responses.items():
                    markdown += f"  - **{status}**: {response.get('description', '')}\n"
                    content = response.get('content', {})
                    for content_type, content_schema in content.items():
                        markdown += f"    - **返回类型**: `{content_type}`\n"
                        schema = content_schema.get('schema', {})
                        if schema:
                            markdown += f"    - **返回结构**:\n"
                            markdown += parse_schema(schema, components)
            markdown += "\n"
    return markdown

# Convert the JSON to Markdown
markdown_content = convert_to_markdown(openapi_data)

# Save to a file
with open("doc/api/openapi.md", "w", encoding='utf-8') as file:
    file.write(markdown_content)

print("OpenAPI documentation has been converted to Markdown format and saved to doc/api/openapi.md")
print("Markdown content preview:")
print(markdown_content[:500])  # Preview first 500 characters
