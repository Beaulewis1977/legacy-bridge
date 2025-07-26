import { NextRequest, NextResponse } from 'next/server';

// Slack webhook endpoint for alerts
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { alert, status, timestamp } = body;

    // In production, this would send to actual Slack webhook
    console.log('Slack alert notification:', {
      alert: alert.title,
      severity: alert.severity,
      status,
      timestamp
    });

    // Simulate Slack webhook response
    return NextResponse.json({
      ok: true,
      message: 'Alert sent to Slack successfully'
    });
  } catch (error) {
    console.error('Failed to send Slack notification:', error);
    return NextResponse.json(
      { error: 'Failed to send Slack notification' },
      { status: 500 }
    );
  }
}