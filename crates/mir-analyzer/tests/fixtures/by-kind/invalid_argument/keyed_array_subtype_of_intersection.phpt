===description===
A TKeyedArray (array shape) should be a subtype of TIntersection when all required
keys are present and their types are compatible. This mirrors the Psalm idiom of
using intersection types for "shape plus extra keys allowed" patterns like:
@psalm-type Context = array<string,mixed> & array{actor:...,target?:...,outcome:...}
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type Context = array<string,mixed> & array{actor: string, target?: string, outcome: string}
 */
class SecurityLoggerTest {
    /**
     * @param Context $context
     */
    public function info(array $context): void {}
    
    public function testValidContext(): void {
        // This should NOT flag - all required keys present with correct types
        $this->info([
            'actor' => 'user123',
            'target' => 'resource456',
            'outcome' => 'success',
            'extra_key' => 'allowed because of array<string,mixed>'
        ]);
    }
    
    public function testValidContextMinimal(): void {
        // This should NOT flag - only required keys, target is optional
        $this->info([
            'actor' => 'user123',
            'outcome' => 'success'
        ]);
    }
    
    public function testMissingRequiredKey(): void {
        // This SHOULD flag - missing required 'actor' key
        $this->info([
            'target' => 'resource456',
            'outcome' => 'success'
        ]);
    }
    
    public function testWrongTypeForRequiredKey(): void {
        // This SHOULD flag - 'actor' should be string, got int
        $this->info([
            'actor' => 123,
            'outcome' => 'success'
        ]);
    }
}
===expect===
InvalidArgument@31:20-34:9: Argument $context of info() expects 'array<string, mixed>&array{'actor': string, 'target'?: string, 'outcome': string}', got 'array{'target': "resource456", 'outcome': "success"}'
InvalidArgument@39:20-42:9: Argument $context of info() expects 'array<string, mixed>&array{'actor': string, 'target'?: string, 'outcome': string}', got 'array{'actor': 123, 'outcome': "success"}'
