// src/app/page.tsx
'use client'

import { useState } from 'react'
import { api } from '@/lib/api'

export default function Home() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<any | null>(null)

  const testFlow = async () => {
    setLoading(true)
    setError(null)
    try {
      // 1. Create first commitment
      const comm1 = await api.createCommitment({ bank_index: 0, amount: 50 })
      console.log('Commitment 1 created:', comm1)

      // 2. Create second commitment
      const comm2 = await api.createCommitment({ bank_index: 1, amount: 30 })
      console.log('Commitment 2 created:', comm2)

      // 3. Start async proof generation
      const proofJob = await api.generateProofAsync({
        required_amount: 60,
        deal_id: 'DEAL123'
      })
      console.log('Proof generation started:', proofJob)

      setResult({ comm1, comm2, proofJob })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Something went wrong')
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="min-h-screen p-8">
      <h1 className="text-2xl font-bold mb-4">RWZ-POF Test</h1>

      <div className="space-y-4">
        <button
          onClick={testFlow}
          disabled={loading}
          className="bg-blue-500 text-white px-4 py-2 rounded disabled:bg-gray-400"
        >
          {loading ? 'Testing...' : 'Run Test Flow'}
        </button>

        {error && (
          <div className="text-red-500">{error}</div>
        )}

        {result && (
          <pre className="bg-gray-100 p-4 rounded overflow-auto">
            {JSON.stringify(result, null, 2)}
          </pre>
        )}
      </div>
    </main>
  )
}