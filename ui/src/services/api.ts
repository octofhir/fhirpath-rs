import {
  FileInfo,
  UploadResponse,
  EvaluationResult,
  AnalysisResult,
  FhirVersion,
  ApiError,
} from './types';

export class FHIRPathAPI {
  private baseUrl: string;

  constructor(baseUrl: string = '') {
    this.baseUrl = baseUrl;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    const data = await response.json();
    
    if (!response.ok) {
      const errorData = data as ApiError;
      throw new Error(errorData.error || `Request failed with status ${response.status}`);
    }

    return data;
  }

  // File operations
  async uploadFile(file: File): Promise<UploadResponse> {
    const formData = new FormData();
    formData.append('file', file);

    const response = await fetch(`${this.baseUrl}/files/upload`, {
      method: 'POST',
      body: formData,
    });

    return this.handleResponse<UploadResponse>(response);
  }

  async getFiles(): Promise<FileInfo[]> {
    const response = await fetch(`${this.baseUrl}/files`);
    return this.handleResponse<FileInfo[]>(response);
  }

  async getFile(filename: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/files/${encodeURIComponent(filename)}`);
    return this.handleResponse<any>(response);
  }

  async deleteFile(filename: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/files/${encodeURIComponent(filename)}`, {
      method: 'DELETE',
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ 
        error: `Failed to delete file: HTTP ${response.status}` 
      })) as ApiError;
      
      throw new Error(errorData.error || 'Failed to delete file');
    }
  }

  // FHIRPath operations
  async evaluate(
    version: FhirVersion,
    expression: string,
    resource?: any,
    filename?: string
  ): Promise<EvaluationResult> {
    const body: any = {
      expression,
    };

    // Include inline resource in body if provided
    if (resource) {
      body.resource = resource;
    }

    // Build URL with filename as query parameter if provided
    let url = `${this.baseUrl}/${version}/evaluate`;
    if (filename) {
      url += `?file=${encodeURIComponent(filename)}`;
    }

    const response = await fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    return this.handleResponse<EvaluationResult>(response);
  }

  async analyze(version: FhirVersion, expression: string): Promise<AnalysisResult> {
    const response = await fetch(`${this.baseUrl}/${version}/analyze`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ expression }),
    });

    return this.handleResponse<AnalysisResult>(response);
  }

  // Health check
  async healthCheck(): Promise<{ status: string; version?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/health`);
      return this.handleResponse<{ status: string; version?: string }>(response);
    } catch (error) {
      throw new Error(`Health check failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }
}

// Singleton instance
export const api = new FHIRPathAPI();