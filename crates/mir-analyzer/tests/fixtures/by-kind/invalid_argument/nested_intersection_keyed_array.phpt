===description===
Test nested intersection types where TKeyedArray appears at multiple levels.
This covers complex intersection patterns with nested array structures.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type BaseConfig = array{debug: bool, environment: string}
 * @psalm-type ExtendedConfig = BaseConfig & array{cache: array{enabled: bool, ttl: int}}
 */
class ConfigTest {
    /**
     * @param ExtendedConfig $config
     */
    public function configure(array $config): void {}
    
    public function testValidConfig(): void {
        // Should NOT flag - all nested structure correct
        $this->configure([
            'debug' => true,
            'environment' => 'production',
            'cache' => [
                'enabled' => true,
                'ttl' => 3600
            ]
        ]);
    }
    
    public function testMissingNestedKey(): void {
        // SHOULD flag - missing 'ttl' in cache
        $this->configure([
            'debug' => false,
            'environment' => 'development',
            'cache' => [
                'enabled' => false
            ]
        ]);
    }
    
    public function testMissingTopLevelKey(): void {
        // SHOULD flag - missing 'environment'
        $this->configure([
            'debug' => true,
            'cache' => [
                'enabled' => true,
                'ttl' => 3600
            ]
        ]);
    }
}
===expect===
InvalidArgument@26:25-32:9: Argument $config of configure() expects 'array{'debug': bool, 'environment': string}&array{'cache': array{'enabled': bool, 'ttl': int}}', got 'array{'debug': false, 'environment': "development", 'cache': array{'enabled': false}}'
InvalidArgument@37:25-43:9: Argument $config of configure() expects 'array{'debug': bool, 'environment': string}&array{'cache': array{'enabled': bool, 'ttl': int}}', got 'array{'debug': true, 'cache': array{'enabled': true, 'ttl': 3600}}'
