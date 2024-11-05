import type { NextConfig } from "next";

/** @type {import('next').NextConfig} */
const nextConfig = {
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://localhost:3030/:path*', // Adjust if your backend port is different
      },
    ]
  }
}

export default nextConfig;
