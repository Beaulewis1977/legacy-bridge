"use client"

import { MainLayout } from "@/components/layout/MainLayout"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Separator } from "@/components/ui/separator"
import { useState } from "react"

export default function DemoPage() {
  const [progress, setProgress] = useState(33)
  const [isEnabled, setIsEnabled] = useState(false)

  return (
    <MainLayout>
      <div className="space-y-8">
        <div>
          <h1 className="text-3xl font-bold">LegacyBridge Components Demo</h1>
          <p className="text-muted-foreground mt-2">
            Showcasing all the shadcn/ui components configured for the project
          </p>
        </div>

        <Separator />

        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {/* Card Example */}
          <Card>
            <CardHeader>
              <CardTitle>Migration Status</CardTitle>
              <CardDescription>Current system migration progress</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center space-x-2">
                <Badge>Active</Badge>
                <Badge variant="secondary">In Progress</Badge>
              </div>
              <Progress value={progress} className="w-full" />
              <p className="text-sm text-muted-foreground">{progress}% Complete</p>
            </CardContent>
            <CardFooter>
              <Button 
                className="w-full"
                onClick={() => setProgress(Math.min(100, progress + 10))}
              >
                Update Progress
              </Button>
            </CardFooter>
          </Card>

          {/* Form Controls */}
          <Card>
            <CardHeader>
              <CardTitle>System Settings</CardTitle>
              <CardDescription>Configure migration parameters</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="system-name">System Name</Label>
                <Input id="system-name" placeholder="Legacy System v1" />
              </div>
              <div className="flex items-center space-x-2">
                <Switch 
                  id="auto-migrate" 
                  checked={isEnabled}
                  onCheckedChange={setIsEnabled}
                />
                <Label htmlFor="auto-migrate">Enable auto-migration</Label>
              </div>
            </CardContent>
          </Card>

          {/* Tabs Example */}
          <Card>
            <CardHeader>
              <CardTitle>Migration Phases</CardTitle>
              <CardDescription>Track different migration stages</CardDescription>
            </CardHeader>
            <CardContent>
              <Tabs defaultValue="analysis" className="w-full">
                <TabsList className="grid w-full grid-cols-3">
                  <TabsTrigger value="analysis">Analysis</TabsTrigger>
                  <TabsTrigger value="migration">Migration</TabsTrigger>
                  <TabsTrigger value="validation">Validation</TabsTrigger>
                </TabsList>
                <TabsContent value="analysis" className="space-y-2">
                  <p className="text-sm">Analyzing legacy codebase...</p>
                  <Badge variant="outline">25 files scanned</Badge>
                </TabsContent>
                <TabsContent value="migration" className="space-y-2">
                  <p className="text-sm">Converting to modern stack...</p>
                  <Badge variant="outline">15 files migrated</Badge>
                </TabsContent>
                <TabsContent value="validation" className="space-y-2">
                  <p className="text-sm">Validating migrated code...</p>
                  <Badge variant="outline">10 tests passed</Badge>
                </TabsContent>
              </Tabs>
            </CardContent>
          </Card>
        </div>

        <Separator />

        {/* Button Variations */}
        <div>
          <h2 className="text-2xl font-semibold mb-4">Button Variants</h2>
          <div className="flex flex-wrap gap-4">
            <Button>Default</Button>
            <Button variant="secondary">Secondary</Button>
            <Button variant="destructive">Destructive</Button>
            <Button variant="outline">Outline</Button>
            <Button variant="ghost">Ghost</Button>
            <Button variant="link">Link</Button>
          </div>
        </div>
      </div>
    </MainLayout>
  )
}