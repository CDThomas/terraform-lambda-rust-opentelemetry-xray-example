data "local_file" "hello_zip" {
  filename = "${path.module}/../target/lambda/hello/bootstrap.zip"
}

resource "aws_lambda_function" "hello" {
  architectures    = ["arm64"]
  filename         = data.local_file.hello_zip.filename
  function_name    = "Hello"
  handler          = "bootstrap"
  layers           = ["arn:aws:lambda:ap-southeast-2:901920570463:layer:aws-otel-collector-arm64-ver-0-74-0:1"]
  role             = aws_iam_role.lambda_exec.arn
  runtime          = "provided.al2"
  source_code_hash = data.local_file.hello_zip.content_base64sha256

  tracing_config {
    mode = "Active"
  }
}

resource "aws_cloudwatch_log_group" "hello" {
  name = "/aws/lambda/${aws_lambda_function.hello.function_name}"

  retention_in_days = 30
}

resource "aws_iam_role" "lambda_exec" {
  name = "serverless_lambda"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Sid    = ""
      Principal = {
        Service = "lambda.amazonaws.com"
      }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_policy" {
  role       = aws_iam_role.lambda_exec.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_function_url" "hello" {
  function_name      = aws_lambda_function.hello.function_name
  authorization_type = "NONE"
}

# X-ray

resource "aws_iam_policy_attachment" "x_ray" {
  name       = "hello-x-ray"
  roles      = [aws_iam_role.lambda_exec.name]
  policy_arn = "arn:aws:iam::aws:policy/AWSXRayDaemonWriteAccess"
}
