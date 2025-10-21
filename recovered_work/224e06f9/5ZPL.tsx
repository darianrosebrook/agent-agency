import React, { useState, useEffect } from 'react';
import { useArbiterVerdict } from '@/hooks/useArbiter';
import { Card, CardHeader, CardTitle, CardContent } from '../shared/Card';
import { Badge } from '../shared/Badge';
import { Button } from '../shared/Button';
import { Alert, AlertDescription } from '../shared/Alert';
import { Input } from '../shared/Input';
import { Textarea } from '../shared/Textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../shared/Select';
import { Play, Pause, Square, Settings, MessageSquare, AlertTriangle, CheckCircle } from 'lucide-react';

interface CliInterventionPanelProps {
  taskId: string;
  onIntervention?: (action: string, params?: any) => void;
}

type InterventionMode = 'strict' | 'auto' | 'dry-run';
type TaskStatus = 'planning' | 'executing' | 'paused' | 'waiting_approval' | 'completed' | 'failed';

export function CliInterventionPanel({ taskId, onIntervention }: CliInterventionPanelProps) {
  const { verdict } = useArbiterVerdict(taskId);
  const [interventionMode, setInterventionMode] = useState<InterventionMode>('auto');
  const [taskStatus, setTaskStatus] = useState<TaskStatus>('planning');
  const [guidanceText, setGuidanceText] = useState('');
  const [overrideVerdict, setOverrideVerdict] = useState('');
  const [overrideReason, setOverrideReason] = useState('');
  const [parameterName, setParameterName] = useState('');
  const [parameterValue, setParameterValue] = useState('');

  // Simulate real-time status updates
  useEffect(() => {
    const statusSequence: TaskStatus[] = ['planning', 'executing', 'waiting_approval', 'executing', 'completed'];
    let index = 0;

    const interval = setInterval(() => {
      setTaskStatus(statusSequence[index % statusSequence.length]);
      index++;
    }, 10000); // Change status every 10 seconds for demo

    return () => clearInterval(interval);
  }, []);

  const handleIntervention = (action: string, params?: any) => {
    console.log(`Intervention: ${action}`, params);
    onIntervention?.(action, params);

    // Reset form fields
    setGuidanceText('');
    setOverrideVerdict('');
    setOverrideReason('');
    setParameterName('');
    setParameterValue('');
  };

  const getStatusColor = (status: TaskStatus) => {
    switch (status) {
      case 'planning': return 'bg-blue-100 text-blue-800';
      case 'executing': return 'bg-green-100 text-green-800';
      case 'paused': return 'bg-yellow-100 text-yellow-800';
      case 'waiting_approval': return 'bg-orange-100 text-orange-800';
      case 'completed': return 'bg-purple-100 text-purple-800';
      case 'failed': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getStatusIcon = (status: TaskStatus) => {
    switch (status) {
      case 'planning': return '🧠';
      case 'executing': return '⚙️';
      case 'paused': return '⏸️';
      case 'waiting_approval': return '⏳';
      case 'completed': return '✅';
      case 'failed': return '❌';
      default: return '❓';
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Settings className="w-5 h-5" />
          CLI Intervention Controls
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Current Status */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-2xl">{getStatusIcon(taskStatus)}</span>
            <div>
              <div className="font-medium">Task Status</div>
              <Badge className={getStatusColor(taskStatus)}>
                {taskStatus.replace('_', ' ').toUpperCase()}
              </Badge>
            </div>
          </div>
          <div className="text-right">
            <div className="text-sm text-gray-600">Intervention Mode</div>
            <Select value={interventionMode} onValueChange={(value: InterventionMode) => setInterventionMode(value)}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="strict">Strict</SelectItem>
                <SelectItem value="auto">Auto</SelectItem>
                <SelectItem value="dry-run">Dry Run</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        {/* Execution Controls */}
        <div className="space-y-4">
          <h3 className="font-medium">Execution Controls</h3>
          <div className="grid grid-cols-2 gap-2">
            {taskStatus === 'paused' && (
              <Button
                onClick={() => handleIntervention('resume')}
                className="flex items-center gap-2"
              >
                <Play className="w-4 h-4" />
                Resume
              </Button>
            )}
            {taskStatus === 'executing' && (
              <Button
                onClick={() => handleIntervention('pause')}
                variant="outline"
                className="flex items-center gap-2"
              >
                <Pause className="w-4 h-4" />
                Pause
              </Button>
            )}
            <Button
              onClick={() => handleIntervention('abort')}
              variant="destructive"
              className="flex items-center gap-2"
            >
              <Square className="w-4 h-4" />
              Abort
            </Button>
          </div>
        </div>

        {/* Arbiter Interventions */}
        {verdict && (
          <div className="space-y-4">
            <h3 className="font-medium">Arbiter Interventions</h3>

            {/* Verdict Override */}
            {verdict.status === 'rejected' || verdict.status === 'waiver_required' && (
              <div className="space-y-2">
                <label className="text-sm font-medium">Override Arbiter Verdict</label>
                <div className="flex gap-2">
                  <Select value={overrideVerdict} onValueChange={setOverrideVerdict}>
                    <SelectTrigger className="flex-1">
                      <SelectValue placeholder="Select verdict" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="approve">Approve</SelectItem>
                      <SelectItem value="reject">Reject</SelectItem>
                    </SelectContent>
                  </Select>
                  <Button
                    onClick={() => handleIntervention('override_verdict', { verdict: overrideVerdict, reason: overrideReason })}
                    disabled={!overrideVerdict || !overrideReason}
                    size="sm"
                  >
                    Override
                  </Button>
                </div>
                <Textarea
                  placeholder="Reason for override..."
                  value={overrideReason}
                  onChange={(e) => setOverrideReason(e.target.value)}
                  className="mt-2"
                />
              </div>
            )}

            {/* Waiver Request */}
            {verdict.status === 'waiver_required' && !verdict.waiverRequired && (
              <Button
                onClick={() => handleIntervention('request_waiver', { reason: 'Manual waiver request via CLI' })}
                variant="outline"
                className="w-full"
              >
                Request Waiver
              </Button>
            )}
          </div>
        )}

        {/* Parameter Modification */}
        <div className="space-y-4">
          <h3 className="font-medium">Parameter Modification</h3>
          <div className="grid grid-cols-2 gap-2">
            <Input
              placeholder="Parameter name"
              value={parameterName}
              onChange={(e) => setParameterName(e.target.value)}
            />
            <Input
              placeholder="New value"
              value={parameterValue}
              onChange={(e) => setParameterValue(e.target.value)}
            />
          </div>
          <Button
            onClick={() => handleIntervention('modify_parameter', { name: parameterName, value: parameterValue })}
            disabled={!parameterName || !parameterValue}
            variant="outline"
            className="w-full"
          >
            Modify Parameter
          </Button>
        </div>

        {/* Guidance Injection */}
        <div className="space-y-4">
          <h3 className="font-medium flex items-center gap-2">
            <MessageSquare className="w-4 h-4" />
            Guidance Injection
          </h3>
          <Textarea
            placeholder="Provide specific guidance for the agent..."
            value={guidanceText}
            onChange={(e) => setGuidanceText(e.target.value)}
            rows={4}
          />
          <Button
            onClick={() => handleIntervention('inject_guidance', { guidance: guidanceText })}
            disabled={!guidanceText.trim()}
            className="w-full"
          >
            Inject Guidance
          </Button>
        </div>

        {/* Mode-Specific Information */}
        <Alert>
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            <strong>Current Mode:</strong> {interventionMode.toUpperCase()}
            {interventionMode === 'strict' && " - Manual approval required for all changes"}
            {interventionMode === 'auto' && " - Automatic execution with quality gates"}
            {interventionMode === 'dry-run' && " - No actual changes will be applied"}
          </AlertDescription>
        </Alert>

        {/* Intervention History */}
        <div className="space-y-2">
          <h3 className="font-medium">Recent Interventions</h3>
          <div className="space-y-1 text-sm text-gray-600">
            <div className="flex items-center gap-2">
              <CheckCircle className="w-3 h-3 text-green-500" />
              <span>Task started with auto mode</span>
              <span className="text-xs text-gray-400">2 min ago</span>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle className="w-3 h-3 text-green-500" />
              <span>Arbiter verdict approved</span>
              <span className="text-xs text-gray-400">1 min ago</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
