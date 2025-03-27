# Test API endpoints for the Jump service

function Test-Endpoint {
    param (
        [string]$Name,
        [string]$Method,
        [string]$Uri,
        [string]$Body = $null,
        [string]$ContentType = "application/json"
    )
    
    Write-Host "===================================================="
    Write-Host "Testing $Name endpoint..."
    Write-Host "Method: $Method"
    Write-Host "URI: $Uri"
    
    if ($Body) {
        Write-Host "Request Body: $Body"
    }
    
    try {
        if ($Body) {
            $response = Invoke-WebRequest -Uri $Uri -Method $Method -Body $Body -ContentType $ContentType -ErrorAction Stop
        } else {
            $response = Invoke-WebRequest -Uri $Uri -Method $Method -ErrorAction Stop
        }
        
        Write-Host "Status: $($response.StatusCode) $($response.StatusDescription)"
        Write-Host "Response Content:"
        $content = $response.Content
        
        # Try to format JSON if possible
        try {
            $jsonContent = $content | ConvertFrom-Json | ConvertTo-Json -Depth 10
            Write-Host $jsonContent
        } catch {
            Write-Host $content
        }
        
        Write-Host "===================================================="
        Write-Host ""
        
        return $response
    } catch {
        Write-Host "Error: $($_.Exception.Message)"
        Write-Host "Status: $($_.Exception.Response.StatusCode) $($_.Exception.Response.StatusDescription)"
        Write-Host "===================================================="
        Write-Host ""
        return $null
    }
}

# Test health endpoint
$healthResponse = Test-Endpoint -Name "Health" -Method "GET" -Uri "http://localhost:8080/api/health"

# Test payload creation
$payload = @{
    content = "Test payload content"
    mime_type = "text/plain"
} | ConvertTo-Json

$createResponse = Test-Endpoint -Name "Create Payload" -Method "POST" -Uri "http://localhost:8080/api/v1/payloads" -Body $payload

# Parse the response to get the payload ID
if ($createResponse) {
    $responseObj = $createResponse.Content | ConvertFrom-Json
    $payloadId = $responseObj.id
    
    if ($payloadId) {
        # Test get payload
        Test-Endpoint -Name "Get Payload" -Method "GET" -Uri "http://localhost:8080/api/v1/payloads/$payloadId"
    }
}
