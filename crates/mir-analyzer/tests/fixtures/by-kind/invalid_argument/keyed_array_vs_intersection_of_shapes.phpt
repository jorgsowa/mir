===description===
Test TKeyedArray against TIntersection of two TKeyedArray shapes.
A shape satisfies an intersection of shapes iff it satisfies each individual shape.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type RequiredFields = array{user_id: int, action: string}
 * @psalm-type OptionalFields = array{user_id: int, action: string, metadata?: array}
 * @psalm-type FullContext = RequiredFields & OptionalFields
 */
class ContextTest {
    /**
     * @param FullContext $ctx
     */
    public function process(array $ctx): void {}
    
    public function testValidFullContext(): void {
        // Should NOT flag - has all required keys
        $this->process([
            'user_id' => 123,
            'action' => 'update',
            'metadata' => ['key' => 'value']
        ]);
    }
    
    public function testValidMinimalContext(): void {
        // Should NOT flag - has required keys, metadata is optional
        $this->process([
            'user_id' => 123,
            'action' => 'delete'
        ]);
    }
    
    public function testMissingUser_id(): void {
        // SHOULD flag - missing required 'user_id'
        $this->process([
            'action' => 'update'
        ]);
    }
    
    public function testMissingAction(): void {
        // SHOULD flag - missing required 'action'
        $this->process([
            'user_id' => 123
        ]);
    }
}
===expect===
InvalidArgument@32:23-34:9: Argument $ctx of process() expects 'array{'user_id': int, 'action': string}&array{'user_id': int, 'action': string, 'metadata'?: array}', got 'array{'action': "update"}'
InvalidArgument@39:23-41:9: Argument $ctx of process() expects 'array{'user_id': int, 'action': string}&array{'user_id': int, 'action': string, 'metadata'?: array}', got 'array{'user_id': 123}'
