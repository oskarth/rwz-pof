'use client'

import { useState, useEffect } from 'react'
import { api } from '@/lib/api'

interface ApiResponse {
  endpoint: string;
  data: any;
  timestamp: string;
}

export default function Home() {
  const [loading, setLoading] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [responses, setResponses] = useState<ApiResponse[]>([])
  const [currentJobId, setCurrentJobId] = useState<string | null>(null)
  const [pollingInterval, setPollingInterval] = useState<NodeJS.Timeout | null>(null)

  useEffect(() => {
    return () => {
      if (pollingInterval) {
        clearInterval(pollingInterval)
      }
    }
  }, [pollingInterval])

  const addResponse = (endpoint: string, data: any) => {
    setResponses(prev => [{
      endpoint,
      data,
      timestamp: new Date().toLocaleTimeString()
    }, ...prev])
  }

  const createCommitment = async (bankIndex: number) => {
    setLoading(`commitment-${bankIndex}`)
    setError(null)
    try {
      const result = await api.createCommitment({
        bank_index: bankIndex,
        amount: bankIndex === 0 ? 50 : 30
      })
      addResponse(`Create Commitment (Bank ${bankIndex})`, result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong')
    } finally {
      setLoading(null)
    }
  }

  const generateProofSync = async () => {
    setLoading('proof-sync')
    setError(null)
    try {
      const result = await api.generateProofSync({
        required_amount: 60,
        deal_id: 'DEAL123'
      })
      addResponse('Generate Proof (Sync)', result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong')
    } finally {
      setLoading(null)
    }
  }

  const generateProofAsync = async () => {
    setLoading('proof-async')
    setError(null)
    try {
      const result = await api.generateProofAsync({
        required_amount: 60,
        deal_id: 'DEAL123'
      })
      setCurrentJobId(result.job_id)
      addResponse('Generate Proof (Async)', result)
      startPolling(result.job_id)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong')
    } finally {
      setLoading(null)
    }
  }

  const checkJobStatus = async (jobId: string) => {
    try {
      const result = await api.checkJobStatus(jobId)
      addResponse('Job Status Check', result)
      if (result.status === 'Completed' || result.status === 'Failed') {
        stopPolling()
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to check job status')
      stopPolling()
    }
  }

  const startPolling = (jobId: string) => {
    if (pollingInterval) {
      clearInterval(pollingInterval)
    }
    const interval = setInterval(() => {
      checkJobStatus(jobId)
    }, 5000)
    setPollingInterval(interval)
  }

  const stopPolling = () => {
    if (pollingInterval) {
      clearInterval(pollingInterval)
      setPollingInterval(null)
    }
    setCurrentJobId(null)
  }

  const verifyProof = async () => {
    setLoading('verify')
    setError(null)
    try {
      const result = await api.verifyProof('DEAL123')
      addResponse('Verify Proof', result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong')
    } finally {
      setLoading(null)
    }
  }

  return (
    <main className="min-h-screen bg-gray-100">
      <div className="max-w-4xl mx-auto p-8">
        <h1 className="text-3xl font-bold mb-8 text-gray-800">RWZ-POF API Tester</h1>

        <div className="space-y-6">
          {/* Commitments */}
          <div className="bg-white p-6 rounded-lg shadow-sm">
            <h2 className="text-xl font-semibold mb-4 text-gray-700">1. Create Commitments</h2>
            <div className="space-x-4">
              <button
                onClick={() => createCommitment(0)}
                disabled={loading === 'commitment-0'}
                className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded disabled:bg-gray-400"
              >
                {loading === 'commitment-0' ? 'Creating...' : 'Create Commitment (Bank 0)'}
              </button>
              <button
                onClick={() => createCommitment(1)}
                disabled={loading === 'commitment-1'}
                className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded disabled:bg-gray-400"
              >
                {loading === 'commitment-1' ? 'Creating...' : 'Create Commitment (Bank 1)'}
              </button>
            </div>
          </div>

          {/* Proof Generation */}
          <div className="bg-white p-6 rounded-lg shadow-sm">
            <h2 className="text-xl font-semibold mb-4 text-gray-700">2. Generate Proof</h2>
            <div className="space-y-4">
              <div className="p-4 bg-gray-50 rounded">
                <h3 className="font-medium mb-3">Option A: Synchronous Flow</h3>
                <button
                  onClick={generateProofSync}
                  disabled={loading === 'proof-sync'}
                  className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded disabled:bg-gray-400"
                >
                  {loading === 'proof-sync' ? 'Generating...' : 'Generate Proof (Sync)'}
                </button>
              </div>

              <div className="p-4 bg-gray-50 rounded">
                <h3 className="font-medium mb-3">Option B: Asynchronous Flow</h3>
                <div className="space-y-2">
                  <button
                    onClick={generateProofAsync}
                    disabled={loading === 'proof-async' || currentJobId !== null}
                    className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded disabled:bg-gray-400"
                  >
                    {loading === 'proof-async' ? 'Generating...' : 'Generate Proof (Async)'}
                  </button>

                  {currentJobId && (
                    <div className="mt-2">
                      <div className="text-sm text-gray-600">
                        Current Job ID: {currentJobId}
                      </div>
                      <div className="text-sm text-gray-600">
                        Status: Auto-checking every 5 seconds...
                      </div>
                      <button
                        onClick={() => checkJobStatus(currentJobId)}
                        className="mt-2 text-sm bg-gray-200 hover:bg-gray-300 px-3 py-1 rounded"
                      >
                        Check Now
                      </button>
                      <button
                        onClick={stopPolling}
                        className="mt-2 ml-2 text-sm bg-gray-200 hover:bg-gray-300 px-3 py-1 rounded"
                      >
                        Stop Checking
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>

          {/* Verify Proof */}
          <div className="bg-white p-6 rounded-lg shadow-sm">
            <h2 className="text-xl font-semibold mb-4 text-gray-700">3. Verify Proof</h2>
            <button
              onClick={verifyProof}
              disabled={loading === 'verify'}
              className="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded disabled:bg-gray-400"
            >
              {loading === 'verify' ? 'Verifying...' : 'Verify Proof'}
            </button>
          </div>

          {/* Error Display */}
          {error && (
            <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
              {error}
            </div>
          )}

          {/* Response History */}
          <div className="bg-white p-6 rounded-lg shadow-sm">
            <h2 className="text-xl font-semibold mb-4 text-gray-700">Response History</h2>
            <div className="space-y-4">
              {responses.map((response, index) => (
                <div key={index} className="border rounded p-4">
                  <div className="flex justify-between items-start mb-2">
                    <span className="font-medium text-black">{response.endpoint}</span>
                    <span className="text-sm text-gray-600">{response.timestamp}</span>
                  </div>
                  <pre className="bg-gray-50 p-3 rounded text-sm overflow-auto text-black">
                    {JSON.stringify(response.data, null, 2)}
                  </pre>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </main>
  )
}