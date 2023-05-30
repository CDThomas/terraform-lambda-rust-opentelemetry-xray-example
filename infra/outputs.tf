
output "function_url" {
  description = "The URL of the function"
  value       = aws_lambda_function_url.hello.function_url
}
