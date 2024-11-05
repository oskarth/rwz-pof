// src/lib/api.ts
const API_BASE_URL = 'http://localhost:3030';

export interface CommitmentRequest {
    bank_index: number;
    amount: number;
}

export interface ProofRequest {
    required_amount: number;
    deal_id: string;
}

export interface ProofResponse {
    deal_info: {
        amount: number;
        buyer: string;
        deal_id: string;
    };
    verified: boolean;
}

export interface AsyncJobResponse {
    status: 'InProgress' | 'Completed';
    created_at: string;
    updated_at: string;
    proof?: {
        verified: boolean;
        deal_info: {
            amount: number;
            buyer: string;
            deal_id: string;
        };
    };
}

export const api = {
    // Create commitment
    async createCommitment(data: CommitmentRequest) {
        const response = await fetch(`${API_BASE_URL}/lb/commitment`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        });

        if (!response.ok) throw new Error('Failed to create commitment');
        return response.json();
    },

    // Generate proof (async)
    async generateProofAsync(data: ProofRequest) {
        const response = await fetch(`${API_BASE_URL}/proofs/async`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        });

        if (!response.ok) throw new Error('Failed to initiate proof generation');
        return response.json();
    },

    // Check proof status
    async checkProofStatus(jobId: string): Promise<AsyncJobResponse> {
        const response = await fetch(`${API_BASE_URL}/proofs/async/${jobId}`);
        if (!response.ok) throw new Error('Failed to check proof status');
        return response.json();
    },

    // Verify proof
    async verifyProof(deal_id: string): Promise<ProofResponse> {
        const response = await fetch(`${API_BASE_URL}/sb/verify`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ deal_id }),
        });

        if (!response.ok) throw new Error('Failed to verify proof');
        return response.json();
    },

    // Add sync proof generation
    async generateProofSync(data: ProofRequest) {
        const response = await fetch(`${API_BASE_URL}/bb/proof`, {  // Note: using /bb/proof for sync
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(data),
        });

        if (!response.ok) throw new Error('Failed to generate proof');
        return response.json();
    },

    // Add job status check
    async checkJobStatus(jobId: string) {
        const response = await fetch(`${API_BASE_URL}/proofs/async/${jobId}`);

        if (!response.ok) throw new Error('Failed to check job status');
        return response.json();
    }

};