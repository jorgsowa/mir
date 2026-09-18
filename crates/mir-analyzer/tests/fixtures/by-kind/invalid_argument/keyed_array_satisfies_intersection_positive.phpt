===description===
Positive test: TKeyedArray should satisfy TIntersection supertype without errors.
This test should have NO expect errors - it verifies the fix works correctly.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type EventContext = array<string,mixed> & array{
 *     actor: string,
 *     target?: string,
 *     outcome: string,
 *     timestamp?: int
 * }
 */
class EventLogger {
    /**
     * @param EventContext $context
     */
    public function log(array $context): void {}
    
    public function testCompleteContext(): void {
        // All required keys present with correct types - should pass
        $this->log([
            'actor' => 'admin',
            'target' => 'user_profile',
            'outcome' => 'updated',
            'timestamp' => 1234567890,
            'extra_field' => 'allowed by array<string,mixed>'
        ]);
    }
    
    public function testMinimalContext(): void {
        // Only required keys, optional keys omitted - should pass
        $this->log([
            'actor' => 'guest',
            'outcome' => 'viewed'
        ]);
    }
    
    public function testWithOneOptional(): void {
        // Required keys plus one optional - should pass
        $this->log([
            'actor' => 'system',
            'target' => 'config',
            'outcome' => 'modified'
        ]);
    }
}
===expect===
