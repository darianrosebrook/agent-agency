#!/usr/bin/env node

/**
 * Test script to verify web interface integration
 */

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

async function testWebInterface() {
  console.log('🧪 Testing Web Interface Integration...\n');

  // Check if web-observer directory exists
  const webObserverPath = path.join(__dirname, 'apps', 'web-observer');
  if (!fs.existsSync(webObserverPath)) {
    console.error('❌ Web observer directory not found');
    return false;
  }

  // Check if package.json exists
  const packageJsonPath = path.join(webObserverPath, 'package.json');
  if (!fs.existsSync(packageJsonPath)) {
    console.error('❌ Web observer package.json not found');
    return false;
  }

  console.log('✅ Web observer directory and package.json found');

  // Try to start the web server briefly to test it works
  return new Promise((resolve) => {
    console.log('🚀 Testing Next.js development server startup...');

    const webServer = spawn('npm', ['run', 'dev'], {
      cwd: webObserverPath,
      stdio: ['ignore', 'pipe', 'pipe'],
      detached: false,
    });

    let startupSuccess = false;
    let outputBuffer = '';

    const timeout = setTimeout(() => {
      if (!startupSuccess) {
        console.error('❌ Web server startup timeout');
        webServer.kill('SIGTERM');
        resolve(false);
      }
    }, 10000);

    webServer.stdout.on('data', (data) => {
      const output = data.toString();
      outputBuffer += output;

      if (output.includes('Ready') || output.includes('started server on') || output.includes('localhost:3000')) {
        startupSuccess = true;
        clearTimeout(timeout);
        console.log('✅ Web server started successfully');

        // Give it a moment then kill it
        setTimeout(() => {
          webServer.kill('SIGTERM');
          resolve(true);
        }, 1000);
      }
    });

    webServer.stderr.on('data', (data) => {
      console.error('Web server stderr:', data.toString());
    });

    webServer.on('exit', (code) => {
      clearTimeout(timeout);
      if (!startupSuccess) {
        console.error(`❌ Web server exited with code ${code}`);
        resolve(false);
      }
    });

    webServer.on('error', (error) => {
      clearTimeout(timeout);
      console.error('❌ Web server failed to start:', error.message);
      resolve(false);
    });
  });
}

async function testMainAppIntegration() {
  console.log('\n🔗 Testing Main Application Integration...');

  // Check if index.ts has the web interface startup code
  const indexPath = path.join(__dirname, 'src', 'index.ts');
  const indexContent = fs.readFileSync(indexPath, 'utf8');

  const hasWebInterfaceImport = indexContent.includes('spawn');
  const hasStartWebInterface = indexContent.includes('startWebInterface');
  const hasWebIntegration = indexContent.includes('await startWebInterface');

  if (!hasWebInterfaceImport) {
    console.error('❌ Missing spawn import in index.ts');
    return false;
  }

  if (!hasStartWebInterface) {
    console.error('❌ Missing startWebInterface function');
    return false;
  }

  if (!hasWebIntegration) {
    console.error('❌ Web interface not integrated in main function');
    return false;
  }

  console.log('✅ Main application integration verified');

  // Check if web-observer is excluded from main TypeScript build
  const tsconfigPath = path.join(__dirname, 'tsconfig.json');
  const tsconfig = JSON.parse(fs.readFileSync(tsconfigPath, 'utf8'));

  if (!tsconfig.exclude || !tsconfig.exclude.includes('apps/web-observer')) {
    console.error('❌ Web observer not excluded from main TypeScript build');
    return false;
  }

  console.log('✅ TypeScript configuration verified');

  return true;
}

async function runTests() {
  try {
    const webInterfaceTest = await testWebInterface();
    const integrationTest = await testMainAppIntegration();

    console.log('\n📊 Test Results:');
    console.log(`Web Interface: ${webInterfaceTest ? '✅ PASS' : '❌ FAIL'}`);
    console.log(`Integration: ${integrationTest ? '✅ PASS' : '❌ FAIL'}`);

    if (webInterfaceTest && integrationTest) {
      console.log('\n🎉 All tests passed! Web interface integration is complete.');
      console.log('\n🚀 To start the full system:');
      console.log('   cd iterations/v2');
      console.log('   npm run dev');
      console.log('\n📱 Then visit: http://localhost:3000');
      return true;
    } else {
      console.log('\n❌ Some tests failed. Please check the errors above.');
      return false;
    }
  } catch (error) {
    console.error('❌ Test execution failed:', error);
    return false;
  }
}

if (require.main === module) {
  runTests().then((success) => {
    process.exit(success ? 0 : 1);
  });
}

module.exports = { testWebInterface, testMainAppIntegration, runTests };
