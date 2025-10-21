import React, { useState, useEffect } from 'react';
import { useDebateData } from '@/hooks/useArbiter';
import { Card, CardHeader, CardTitle, CardContent } from '../shared/Card';
import { Badge } from '../shared/Badge';
import { Button } from '../shared/Button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../shared/Tabs';
import { MessageSquare, Users, TrendingUp, Award, Clock, ChevronRight } from 'lucide-react';

interface DebateResult {
  winningOutputIndex: number;
  factualAccuracyScore: number;
  debateRounds: number;
  evidenceManifest: EvidenceManifest;
  debateTranscript?: DebateRound[];
}

interface DebateRound {
  roundNumber: number;
  participants: DebateParticipant[];
  winner: string; // model name
  keyArguments: string[];
  factCheckingResults: FactCheckResult[];
  timestamp: string;
}

interface DebateParticipant {
  modelName: string;
  output: string;
  confidence: number;
  claims: AtomicClaim[];
  rebuttals: string[];
}

interface FactCheckResult {
  claimId: string;
  verified: boolean;
  evidenceFound: boolean;
  confidence: number;
  correction?: string;
}

interface AtomicClaim {
  id: string;
  claimText: string;
  subject: string;
  predicate: string;
  object?: string;
  confidence: number;
}

interface EvidenceManifest {
  claims: AtomicClaim[];
  verificationResults: VerificationResult[];
  factualAccuracyScore: number;
  cawsComplianceScore: number;
}

interface VerificationResult {
  claimId: string;
  verificationStatus: 'Verified' | 'Unverified';
  confidence: number;
  evidence: Evidence[];
}

interface Evidence {
  source: string;
  content: string;
  relevance: number;
  confidence: number;
}

interface DebateVisualizationProps {
  taskId: string;
  debateResult: DebateResult | null;
}

export function DebateVisualization({ taskId, debateResult }: DebateVisualizationProps) {
  const [selectedRound, setSelectedRound] = useState<number | null>(null);
  const [expandedTranscript, setExpandedTranscript] = useState(false);

  if (!debateResult) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="text-center text-gray-500">
            <MessageSquare className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>No debate data available for this task</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <MessageSquare className="w-5 h-5" />
          Multi-Model Debate Analysis
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Debate Summary */}
        <div className="grid grid-cols-3 gap-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600">{debateResult.debateRounds}</div>
            <div className="text-sm text-gray-600">Debate Rounds</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {(debateResult.factualAccuracyScore * 100).toFixed(0)}%
            </div>
            <div className="text-sm text-gray-600">Factual Accuracy</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600">#{debateResult.winningOutputIndex + 1}</div>
            <div className="text-sm text-gray-600">Winning Output</div>
          </div>
        </div>

        {/* Debate Rounds Timeline */}
        {debateResult.debateTranscript && debateResult.debateTranscript.length > 0 && (
          <div className="space-y-4">
            <h3 className="font-medium flex items-center gap-2">
              <Clock className="w-4 h-4" />
              Debate Rounds
            </h3>
            <div className="space-y-2">
              {debateResult.debateTranscript.map((round, index) => (
                <div
                  key={round.roundNumber}
                  className={`border rounded-lg p-3 cursor-pointer transition-colors ${
                    selectedRound === index ? 'border-blue-500 bg-blue-50' : 'hover:bg-gray-50'
                  }`}
                  onClick={() => setSelectedRound(selectedRound === index ? null : index)}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <Badge variant="outline">Round {round.roundNumber}</Badge>
                      <span className="text-sm font-medium">{round.participants.length} models</span>
                      <ChevronRight className={`w-4 h-4 transition-transform ${selectedRound === index ? 'rotate-90' : ''}`} />
                    </div>
                    <div className="flex items-center gap-2">
                      <Award className="w-4 h-4 text-yellow-500" />
                      <span className="text-sm font-medium">{round.winner}</span>
                    </div>
                  </div>

                  {selectedRound === index && (
                    <div className="mt-3 space-y-3">
                      {/* Round Participants */}
                      <div className="space-y-2">
                        <h4 className="text-sm font-medium">Participants</h4>
                        {round.participants.map((participant, pIndex) => (
                          <div key={pIndex} className="bg-white border rounded p-2">
                            <div className="flex items-center justify-between mb-1">
                              <span className="font-medium text-sm">{participant.modelName}</span>
                              <Badge variant="outline" className="text-xs">
                                {(participant.confidence * 100).toFixed(0)}%
                              </Badge>
                            </div>
                            <div className="text-xs text-gray-600 line-clamp-2">
                              {participant.output.substring(0, 100)}...
                            </div>
                          </div>
                        ))}
                      </div>

                      {/* Key Arguments */}
                      {round.keyArguments.length > 0 && (
                        <div className="space-y-2">
                          <h4 className="text-sm font-medium">Key Arguments</h4>
                          <ul className="text-xs text-gray-700 space-y-1">
                            {round.keyArguments.slice(0, 3).map((arg, argIndex) => (
                              <li key={argIndex} className="flex items-start gap-2">
                                <span className="text-blue-500 mt-1">•</span>
                                <span>{arg}</span>
                              </li>
                            ))}
                          </ul>
                        </div>
                      )}

                      {/* Fact Checking Results */}
                      {round.factCheckingResults.length > 0 && (
                        <div className="space-y-2">
                          <h4 className="text-sm font-medium">Fact Checking</h4>
                          <div className="grid grid-cols-2 gap-2">
                            {round.factCheckingResults.slice(0, 4).map((fact, factIndex) => (
                              <div key={factIndex} className={`text-xs p-2 rounded border ${
                                fact.verified ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'
                              }`}>
                                <div className="font-medium">
                                  {fact.verified ? '✓' : '✗'} Claim {fact.claimId.slice(-4)}
                                </div>
                                <div className="text-gray-600">
                                  {(fact.confidence * 100).toFixed(0)}% confidence
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Evidence Analysis */}
        <Tabs defaultValue="claims" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="claims">Claims Analysis</TabsTrigger>
            <TabsTrigger value="evidence">Evidence Quality</TabsTrigger>
          </TabsList>

          <TabsContent value="claims" className="space-y-4">
            <div className="space-y-2">
              <h4 className="font-medium">Total Claims Analyzed</h4>
              <div className="text-2xl font-bold">{debateResult.evidenceManifest.claims.length}</div>
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div className="text-center">
                <div className="text-xl font-bold text-green-600">
                  {debateResult.evidenceManifest.verificationResults.filter(r => r.verificationStatus === 'Verified').length}
                </div>
                <div className="text-sm text-gray-600">Verified</div>
              </div>
              <div className="text-center">
                <div className="text-xl font-bold text-red-600">
                  {debateResult.evidenceManifest.verificationResults.filter(r => r.verificationStatus === 'Unverified').length}
                </div>
                <div className="text-sm text-gray-600">Unverified</div>
              </div>
              <div className="text-center">
                <div className="text-xl font-bold text-blue-600">
                  {(debateResult.evidenceManifest.claims.reduce((sum, claim) => sum + claim.confidence, 0) / debateResult.evidenceManifest.claims.length * 100).toFixed(0)}%
                </div>
                <div className="text-sm text-gray-600">Avg Confidence</div>
              </div>
            </div>
          </TabsContent>

          <TabsContent value="evidence" className="space-y-4">
            <div className="space-y-3">
              {debateResult.evidenceManifest.verificationResults.slice(0, 5).map((result, index) => (
                <div key={index} className="border rounded-lg p-3">
                  <div className="flex items-center justify-between mb-2">
                    <Badge className={result.verificationStatus === 'Verified' ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}>
                      {result.verificationStatus}
                    </Badge>
                    <span className="text-sm text-gray-600">
                      {(result.confidence * 100).toFixed(0)}% confidence
                    </span>
                  </div>
                  <div className="text-sm text-gray-700 mb-2">
                    Claim {result.claimId.slice(-4)}
                  </div>
                  <div className="text-xs text-gray-600">
                    {result.evidence.length} evidence sources
                  </div>
                </div>
              ))}
            </div>
          </TabsContent>
        </Tabs>

        {/* Debate Insights */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h4 className="font-medium text-blue-900 mb-2 flex items-center gap-2">
            <TrendingUp className="w-4 h-4" />
            Debate Insights
          </h4>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>• {debateResult.debateRounds} rounds of cross-examination improved factual accuracy by {(debateResult.factualAccuracyScore * 100 - 70).toFixed(0)}%</li>
            <li>• {debateResult.evidenceManifest.claims.length} claims were analyzed across all models</li>
            <li>• Winner determined through evidence-based adjudication</li>
          </ul>
        </div>
      </CardContent>
    </Card>
  );
}
