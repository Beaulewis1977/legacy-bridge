import { NextRequest, NextResponse } from 'next/server';
import { AlertManager } from '@/lib/monitoring/alert-manager';

// Initialize alert manager (in production, this would be a singleton)
const alertManager = new AlertManager();

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const type = searchParams.get('type') || 'active';
    const limit = parseInt(searchParams.get('limit') || '100', 10);

    let alerts;
    switch (type) {
      case 'active':
        alerts = alertManager.getActiveAlerts();
        break;
      case 'history':
        alerts = alertManager.getAlertHistory(limit);
        break;
      case 'rules':
        return NextResponse.json({ rules: alertManager.getRules() });
      default:
        return NextResponse.json(
          { error: 'Invalid alert type' },
          { status: 400 }
        );
    }

    return NextResponse.json({ alerts });
  } catch (error) {
    console.error('Failed to get alerts:', error);
    return NextResponse.json(
      { error: 'Failed to retrieve alerts' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { action, alertId, rule } = body;

    switch (action) {
      case 'acknowledge':
        if (!alertId) {
          return NextResponse.json(
            { error: 'Alert ID required' },
            { status: 400 }
          );
        }
        const acknowledged = alertManager.acknowledgeAlert(alertId);
        return NextResponse.json({ success: acknowledged });

      case 'add_rule':
        if (!rule) {
          return NextResponse.json(
            { error: 'Rule definition required' },
            { status: 400 }
          );
        }
        alertManager.addRule(rule);
        return NextResponse.json({ success: true });

      case 'remove_rule':
        if (!body.ruleId) {
          return NextResponse.json(
            { error: 'Rule ID required' },
            { status: 400 }
          );
        }
        alertManager.removeRule(body.ruleId);
        return NextResponse.json({ success: true });

      default:
        return NextResponse.json(
          { error: 'Invalid action' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Failed to process alert action:', error);
    return NextResponse.json(
      { error: 'Failed to process alert action' },
      { status: 500 }
    );
  }
}