const crypto = require('crypto');
const fs = require('fs');

// Test the checksum calculation (similar to what's in security-provenance.ts)
function calculateFileHash(filePath, algorithm = 'sha256') {
  try {
    const fileContent = fs.readFileSync(filePath);
    return crypto.createHash(algorithm).update(fileContent).digest('hex');
  } catch (error) {
    console.error(`Failed to calculate ${algorithm} hash for ${filePath}:`, error.message);
    throw error;
  }
}

// Test the checksums
console.log('Testing checksum calculation...');
const sha256 = calculateFileHash('test_checksum.txt', 'sha256');
const sha512 = calculateFileHash('test_checksum.txt', 'sha512');

console.log('SHA256:', sha256);
console.log('SHA512:', sha512);

// Test with a different file to see if checksums change
fs.writeFileSync('test_checksum2.txt', 'This is a different test file.');
const sha256_2 = calculateFileHash('test_checksum2.txt', 'sha256');
console.log('SHA256 (different file):', sha256_2);
console.log('Checksums are different:', sha256 !== sha256_2);

// Test error handling
try {
  calculateFileHash('nonexistent.txt');
} catch (error) {
  console.log('Error handling works:', error.message);
}
