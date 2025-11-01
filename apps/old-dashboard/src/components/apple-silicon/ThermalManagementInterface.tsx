/**
 * ThermalManagementInterface Component
 * Real-time temperature monitoring and cooling system controls
 *
 * @author @darianrosebrook
 */

'use client';

import { useState, useEffect, useMemo } from 'react';
import { Text } from '@/design-system/primitives';
import { Button } from '@/design-system/primitives';
import { Badge } from '@/design-system/primitives';
import { Progress } from '@/design-system/primitives';
import {
  Thermometer,
  Fan,
  Settings,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  TrendingUp,
  TrendingDown,
  Zap,
  Activity,
  Clock,
  Gauge,
  Wind,
  Flame
} from 'lucide-react';
import { appleSiliconApiClient } from '@/lib/apple-silicon-api';
import { useAppleSiliconWebSocket, useRealTimeThermalMonitoring } from '@/hooks/useAppleSiliconWebSocket';
import { useAppleSiliconStore, useAppleSiliconActions } from '@/stores/apple-silicon';
import styles from './ThermalManagementInterface.module.scss';

// Thermal data interfaces
interface ThermalSensor {
  name: string;
  temperature: number;
  threshold: {
    warning: number;
    critical: number;
  };
  status: 'normal' | 'warning' | 'critical';
  location: string;
}

interface FanControl {
  id: string;
  name: string;
  currentSpeed: number;
  maxSpeed: number;
  minSpeed: number;
  mode: 'auto' | 'manual';
  status: 'active' | 'idle';
}

interface ThermalEvent {
  id: string;
  timestamp: Date;
  type: 'temperature_spike' | 'throttling_activated' | 'cooling_activated' | 'thermal_shutdown';
  severity: 'low' | 'medium' | 'high' | 'critical';
  temperature: number;
  component: string;
  message: string;
  resolved: boolean;
}

interface ThermalPolicy {
  id: string;
  name: string;
  description: string;
  maxTemperature: number;
  fanCurve: 'quiet' | 'balanced' | 'performance';
  throttlingEnabled: boolean;
  active: boolean;
}

export function ThermalManagementInterface() {
  // State management
  const [thermalSensors, setThermalSensors] = useState<ThermalSensor[]>([]);
  const [fans, setFans] = useState<FanControl[]>([]);
  const [thermalEvents, setThermalEvents] = useState<ThermalEvent[]>([]);
  const [policies, setPolicies] = useState<ThermalPolicy[]>([]);
  const [activePolicy, setActivePolicy] = useState<string>('');
  const [viewMode, setViewMode] = useState<'monitoring' | 'controls' | 'policies' | 'events'>('monitoring');
  const [refreshing, setRefreshing] = useState(false);

  // Real-time data hooks
  const { isConnected } = useAppleSiliconWebSocket();
  const { thermalMetrics, isOverheating } = useRealTimeThermalMonitoring();

  // Fetch thermal data
  const fetchThermalData = async () => {
    try {
      setRefreshing(true);

      // Get thermal policies
      const thermalPolicies = await appleSiliconApiClient.getThermalPolicies();
      setPolicies(thermalPolicies);

      // Get thermal events
      const events = await appleSiliconApiClient.getThermalEvents(20);
      setThermalEvents(events.map(event => ({
        ...event,
        timestamp: new Date(event.timestamp)
      })));

      // Mock thermal sensors data (would come from API)
      const mockSensors: ThermalSensor[] = [
        {
          name: 'CPU Core',
          temperature: thermalMetrics?.cpuTemperature || 45.2,
          threshold: { warning: 75, critical: 85 },
          status: thermalMetrics && thermalMetrics.cpuTemperature > 85 ? 'critical' :
                  thermalMetrics && thermalMetrics.cpuTemperature > 75 ? 'warning' : 'normal',
          location: 'Main Processor'
        },
        {
          name: 'GPU Core',
          temperature: thermalMetrics?.gpuTemperature || 42.8,
          threshold: { warning: 70, critical: 80 },
          status: thermalMetrics && thermalMetrics.gpuTemperature > 80 ? 'critical' :
                  thermalMetrics && thermalMetrics.gpuTemperature > 70 ? 'warning' : 'normal',
          location: 'Graphics Processor'
        },
        {
          name: 'ANE Core',
          temperature: thermalMetrics?.aneTemperature || 38.5,
          threshold: { warning: 65, critical: 75 },
          status: thermalMetrics && thermalMetrics.aneTemperature > 75 ? 'critical' :
                  thermalMetrics && thermalMetrics.aneTemperature > 65 ? 'warning' : 'normal',
          location: 'Neural Engine'
        },
        {
          name: 'System Ambient',
          temperature: thermalMetrics?.ambientTemperature || 28.3,
          threshold: { warning: 35, critical: 40 },
          status: thermalMetrics && thermalMetrics.ambientTemperature > 40 ? 'critical' :
                  thermalMetrics && thermalMetrics.ambientTemperature > 35 ? 'warning' : 'normal',
          location: 'System Board'
        }
      ];

      setThermalSensors(mockSensors);

      // Mock fan data (would come from API)
      const mockFans: FanControl[] = [
        {
          id: 'fan1',
          name: 'Primary Fan',
          currentSpeed: thermalMetrics?.fanSpeed || 1800,
          maxSpeed: 6000,
          minSpeed: 1000,
          mode: 'auto',
          status: 'active'
        }
      ];

      setFans(mockFans);

      // Set active policy
      const active = thermalPolicies.find(p => p.active);
      if (active) {
        setActivePolicy(active.id);
      }

    } catch (err) {
      console.error('Failed to fetch thermal data:', err);
    } finally {
      setRefreshing(false);
    }
  };

  // Handle fan speed change
  const handleFanSpeedChange = async (fanId: string, newSpeed: number) => {
    try {
      // Update local state immediately for responsive UI
      setFans(prev => prev.map(fan =>
        fan.id === fanId ? { ...fan, currentSpeed: newSpeed, mode: 'manual' } : fan
      ));

      // TODO: Send to API
      console.log(`Setting fan ${fanId} to ${newSpeed} RPM`);
    } catch (err) {
      console.error('Failed to update fan speed:', err);
      // Revert on error
      fetchThermalData();
    }
  };

  // Handle policy change
  const handlePolicyChange = async (policyId: string) => {
    try {
      await appleSiliconApiClient.setThermalPolicy(policyId);
      setActivePolicy(policyId);

      // Refresh data to get updated policy status
      await fetchThermalData();
    } catch (err) {
      console.error('Failed to change thermal policy:', err);
    }
  };

  // Handle manual thermal override
  const handleThermalOverride = async (settings: {
    maxTemperature?: number;
    fanMode?: 'quiet' | 'balanced' | 'performance';
  }) => {
    try {
      await appleSiliconApiClient.overrideThermalSettings(settings);
      await fetchThermalData();
    } catch (err) {
      console.error('Failed to override thermal settings:', err);
    }
  };

  // Handle refresh
  const handleRefresh = async () => {
    await fetchThermalData();
  };

  // Calculate thermal status
  const thermalStatus = useMemo(() => {
    const criticalCount = thermalSensors.filter(s => s.status === 'critical').length;
    const warningCount = thermalSensors.filter(s => s.status === 'warning').length;

    if (criticalCount > 0) return { level: 'critical', color: 'error', icon: Flame };
    if (warningCount > 0 || isOverheating) return { level: 'warning', color: 'warning', icon: AlertTriangle };
    return { level: 'normal', color: 'success', icon: CheckCircle };
  }, [thermalSensors, isOverheating]);

  // Get temperature color
  const getTemperatureColor = (temperature: number, warning: number, critical: number) => {
    if (temperature >= critical) return 'var(--color-error)';
    if (temperature >= warning) return 'var(--color-warning)';
    return 'var(--color-success)';
  };

  // Format temperature
  const formatTemperature = (temp: number) => `${temp.toFixed(1)}°C`;

  // Initial data load
  useEffect(() => {
    fetchThermalData();
  }, []);

  // View mode options
  const viewModeOptions = [
    { value: 'monitoring', label: 'Monitoring', icon: Thermometer },
    { value: 'controls', label: 'Controls', icon: Settings },
    { value: 'policies', label: 'Policies', icon: Gauge },
    { value: 'events', label: 'Events', icon: Clock },
  ];

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerLeft}>
          <Text variant="h3">Thermal Management</Text>
          <Text variant="paragraph-small" color="secondary">
            Monitor temperature and control cooling systems
          </Text>
        </div>

        <div className={styles.headerRight}>
          {/* Thermal Status */}
          <div className={styles.thermalStatus}>
            <thermalStatus.icon
              size={16}
              style={{ color: `var(--color-${thermalStatus.color})` }}
            />
            <span style={{ color: `var(--color-${thermalStatus.color})` }}>
              {thermalStatus.level}
            </span>
          </div>

          {/* Connection Status */}
          <div className={styles.connectionStatus}>
            {isConnected ? (
              <Activity size={12} className={styles.connected} />
            ) : (
              <Clock size={12} className={styles.disconnected} />
            )}
          </div>

          <Button
            variant="secondary"
            size="sm"
            onClick={handleRefresh}
            disabled={refreshing}
          >
            <RefreshCw
              size={16}
              className={refreshing ? styles.spinning : ''}
            />
            Refresh
          </Button>
        </div>
      </div>

      {/* View Mode Tabs */}
      <div className={styles.viewTabs}>
        {viewModeOptions.map((option) => (
          <button
            key={option.value}
            onClick={() => setViewMode(option.value as any)}
            className={`${styles.viewTab} ${viewMode === option.value ? styles.active : ''}`}
          >
            <option.icon size={16} />
            <span>{option.label}</span>
          </button>
        ))}
      </div>

      {/* Monitoring View */}
      {viewMode === 'monitoring' && (
        <div className={styles.monitoring}>
          {/* Temperature Sensors */}
          <div className={styles.sensorsGrid}>
            {thermalSensors.map((sensor) => (
              <div key={sensor.name} className={styles.sensorCard}>
                <div className={styles.sensorHeader}>
                  <Thermometer className={styles.sensorIcon} />
                  <div className={styles.sensorInfo}>
                    <Text variant="h5" className={styles.sensorName}>
                      {sensor.name}
                    </Text>
                    <Text variant="paragraph-small" color="secondary">
                      {sensor.location}
                    </Text>
                  </div>
                  <Badge
                    variant={
                      sensor.status === 'critical' ? 'error' :
                      sensor.status === 'warning' ? 'warning' : 'success'
                    }
                    size="sm"
                  >
                    {sensor.status}
                  </Badge>
                </div>

                <div className={styles.temperatureDisplay}>
                  <Text
                    variant="display-2"
                    style={{ color: getTemperatureColor(sensor.temperature, sensor.threshold.warning, sensor.threshold.critical) }}
                  >
                    {formatTemperature(sensor.temperature)}
                  </Text>

                  <div className={styles.temperatureBar}>
                    <div
                      className={styles.temperatureProgress}
                      style={{
                        width: `${Math.min((sensor.temperature / sensor.threshold.critical) * 100, 100)}%`,
                        backgroundColor: getTemperatureColor(sensor.temperature, sensor.threshold.warning, sensor.threshold.critical)
                      }}
                    />
                  </div>

                  <div className={styles.temperatureThresholds}>
                    <span>0°C</span>
                    <span className={styles.warningThreshold}>
                      {sensor.threshold.warning}°C
                    </span>
                    <span className={styles.criticalThreshold}>
                      {sensor.threshold.critical}°C
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Fan Status */}
          <div className={styles.fanStatus}>
            <div className={styles.fanCard}>
              <div className={styles.fanHeader}>
                <Wind className={styles.fanIcon} />
                <Text variant="h4">Cooling System</Text>
              </div>

              <div className={styles.fanGrid}>
                {fans.map((fan) => (
                  <div key={fan.id} className={styles.fanItem}>
                    <div className={styles.fanInfo}>
                      <Text variant="paragraph-medium" className={styles.fanName}>
                        {fan.name}
                      </Text>
                      <Badge variant={fan.status === 'active' ? 'success' : 'secondary'} size="sm">
                        {fan.status}
                      </Badge>
                    </div>

                    <div className={styles.fanSpeed}>
                      <Text variant="h3">{fan.currentSpeed.toLocaleString()}</Text>
                      <Text variant="paragraph-small" color="secondary">RPM</Text>
                    </div>

                    <div className={styles.fanMode}>
                      <Badge variant="secondary" size="sm">
                        {fan.mode === 'auto' ? 'Auto' : 'Manual'}
                      </Badge>
                    </div>

                    <Progress
                      value={(fan.currentSpeed / fan.maxSpeed) * 100}
                      size="sm"
                      className={styles.fanProgress}
                    />
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Thermal Overview */}
          <div className={styles.thermalOverview}>
            <div className={styles.overviewCard}>
              <Text variant="h4">Thermal Overview</Text>

              <div className={styles.overviewGrid}>
                <div className={styles.overviewItem}>
                  <Text variant="paragraph-small" color="secondary">Cooling Mode</Text>
                  <Badge variant="info">
                    {thermalMetrics?.coolingMode || 'Active'}
                  </Badge>
                </div>

                <div className={styles.overviewItem}>
                  <Text variant="paragraph-small" color="secondary">Thermal Pressure</Text>
                  <Badge
                    variant={
                      thermalMetrics?.thermalPressure === 'critical' ? 'error' :
                      thermalMetrics?.thermalPressure === 'heavy' ? 'warning' : 'success'
                    }
                  >
                    {thermalMetrics?.thermalPressure || 'nominal'}
                  </Badge>
                </div>

                <div className={styles.overviewItem}>
                  <Text variant="paragraph-small" color="secondary">Recent Events</Text>
                  <Text variant="h4">{thermalEvents.length}</Text>
                </div>

                <div className={styles.overviewItem}>
                  <Text variant="paragraph-small" color="secondary">Active Policy</Text>
                  <Text variant="paragraph-medium">
                    {policies.find(p => p.active)?.name || 'Default'}
                  </Text>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Controls View */}
      {viewMode === 'controls' && (
        <div className={styles.controls}>
          <div className={styles.controlsCard}>
            <Text variant="h4">Manual Controls</Text>
            <Text variant="paragraph-small" color="secondary">
              Override automatic thermal management (use with caution)
            </Text>

            <div className={styles.controlSection}>
              <Text variant="h5">Fan Speed Control</Text>
              {fans.map((fan) => (
                <div key={fan.id} className={styles.fanControl}>
                  <div className={styles.fanControlHeader}>
                    <Text variant="paragraph-medium">{fan.name}</Text>
                    <div className={styles.fanSpeedDisplay}>
                      <Text variant="h3">{fan.currentSpeed}</Text>
                      <Text variant="paragraph-small" color="secondary">RPM</Text>
                    </div>
                  </div>

                  <div className={styles.fanSpeedSlider}>
                    <input
                      type="range"
                      min={fan.minSpeed}
                      max={fan.maxSpeed}
                      value={fan.currentSpeed}
                      onChange={(e) => handleFanSpeedChange(fan.id, parseInt(e.target.value))}
                      className={styles.slider}
                    />
                    <div className={styles.sliderLabels}>
                      <span>{fan.minSpeed}</span>
                      <span>{fan.maxSpeed}</span>
                    </div>
                  </div>

                  <div className={styles.fanModeButtons}>
                    <Button
                      variant={fan.mode === 'auto' ? 'primary' : 'secondary'}
                      size="sm"
                      onClick={() => handleFanSpeedChange(fan.id, fan.currentSpeed)}
                    >
                      Auto
                    </Button>
                    <Button
                      variant={fan.mode === 'manual' ? 'primary' : 'secondary'}
                      size="sm"
                      onClick={() => handleFanSpeedChange(fan.id, fan.currentSpeed)}
                    >
                      Manual
                    </Button>
                  </div>
                </div>
              ))}
            </div>

            <div className={styles.controlSection}>
              <Text variant="h5">Temperature Limits</Text>
              <div className={styles.tempLimitControls}>
                <Button
                  variant="outline"
                  onClick={() => handleThermalOverride({ maxTemperature: 80 })}
                >
                  Conservative (80°C)
                </Button>
                <Button
                  variant="outline"
                  onClick={() => handleThermalOverride({ maxTemperature: 85 })}
                >
                  Balanced (85°C)
                </Button>
                <Button
                  variant="outline"
                  onClick={() => handleThermalOverride({ maxTemperature: 90 })}
                >
                  Performance (90°C)
                </Button>
              </div>
            </div>

            <div className={styles.controlWarning}>
              <AlertTriangle className={styles.warningIcon} />
              <div>
                <Text variant="paragraph-medium" className={styles.warningTitle}>
                  Manual Control Warning
                </Text>
                <Text variant="paragraph-small" color="secondary">
                  Manual thermal controls bypass automatic safety systems.
                  Use only for testing or in controlled environments.
                </Text>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Policies View */}
      {viewMode === 'policies' && (
        <div className={styles.policies}>
          <div className={styles.policiesGrid}>
            {policies.map((policy) => (
              <div key={policy.id} className={styles.policyCard}>
                <div className={styles.policyHeader}>
                  <div className={styles.policyInfo}>
                    <Text variant="h5">{policy.name}</Text>
                    <Text variant="paragraph-small" color="secondary">
                      {policy.description}
                    </Text>
                  </div>
                  {policy.active && (
                    <Badge variant="success">Active</Badge>
                  )}
                </div>

                <div className={styles.policyDetails}>
                  <div className={styles.policyMetric}>
                    <Text variant="paragraph-small" color="secondary">Max Temperature</Text>
                    <Text variant="paragraph-medium">{policy.maxTemperature}°C</Text>
                  </div>

                  <div className={styles.policyMetric}>
                    <Text variant="paragraph-small" color="secondary">Fan Curve</Text>
                    <Badge variant="secondary" size="sm">
                      {policy.fanCurve}
                    </Badge>
                  </div>

                  <div className={styles.policyMetric}>
                    <Text variant="paragraph-small" color="secondary">Throttling</Text>
                    <Badge variant={policy.throttlingEnabled ? 'success' : 'error'} size="sm">
                      {policy.throttlingEnabled ? 'Enabled' : 'Disabled'}
                    </Badge>
                  </div>
                </div>

                {!policy.active && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handlePolicyChange(policy.id)}
                    className={styles.activateButton}
                  >
                    Activate Policy
                  </Button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Events View */}
      {viewMode === 'events' && (
        <div className={styles.events}>
          <div className={styles.eventsCard}>
            <div className={styles.eventsHeader}>
              <Text variant="h4">Thermal Events</Text>
              <Text variant="paragraph-small" color="secondary">
                Recent temperature and cooling system events
              </Text>
            </div>

            <div className={styles.eventsList}>
              {thermalEvents.map((event) => (
                <div key={event.id} className={styles.eventItem}>
                  <div className={styles.eventIcon}>
                    {event.type === 'temperature_spike' && <Flame size={16} />}
                    {event.type === 'throttling_activated' && <Zap size={16} />}
                    {event.type === 'cooling_activated' && <Wind size={16} />}
                    {event.type === 'thermal_shutdown' && <AlertTriangle size={16} />}
                  </div>

                  <div className={styles.eventContent}>
                    <div className={styles.eventHeader}>
                      <Text variant="paragraph-medium" className={styles.eventMessage}>
                        {event.message}
                      </Text>
                      <Badge
                        variant={
                          event.severity === 'critical' ? 'error' :
                          event.severity === 'high' ? 'warning' :
                          event.severity === 'medium' ? 'warning' : 'secondary'
                        }
                        size="sm"
                      >
                        {event.severity}
                      </Badge>
                    </div>

                    <div className={styles.eventDetails}>
                      <Text variant="paragraph-small" color="secondary">
                        {event.component} • {formatTemperature(event.temperature)} • {event.timestamp.toLocaleString()}
                      </Text>
                      {event.resolved && (
                        <Badge variant="success" size="sm">Resolved</Badge>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
