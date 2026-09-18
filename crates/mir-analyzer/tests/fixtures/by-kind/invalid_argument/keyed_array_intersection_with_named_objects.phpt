===description===
Test TKeyedArray against TIntersection that includes both array types and named objects.
This covers the case where an intersection has multiple array bounds plus interface bounds.
===config===
suppress=UnusedParam
===file===
<?php
interface Loggable {
    public function getLogMessage(): string;
}

interface Timestamped {
    public function getTimestamp(): int;
}

/**
 * @psalm-type LogEntry = array{message: string, level: string} & Loggable & Timestamped
 */
class LoggerTest {
    /**
     * @param LogEntry $entry
     */
    public function log(array $entry): void {}
    
    public function testValidEntry(): void {
        // Should NOT flag - array shape with required keys
        // Note: In practice, this would also need the object implementing the interfaces
        // but structurally the array shape part should match
        $this->log([
            'message' => 'Test message',
            'level' => 'info'
        ]);
    }
    
    public function testMissingKey(): void {
        // SHOULD flag - missing 'level' key
        $this->log([
            'message' => 'Test message'
        ]);
    }
}
===expect===
InvalidArgument@31:19-33:9: Argument $entry of log() expects 'array{'message': string, 'level': string}&Loggable&Timestamped', got 'array{'message': "Test message"}'
