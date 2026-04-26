===file:composer.json===
{"autoload":{"psr-4":{"App\\":"src/"}}}
===file:src/Config.php===
<?php
namespace App;

class Config {
    private array $data;

    public function __construct(array $data) {
        $this->data = $data;
    }

    /**
     * @throws \InvalidArgumentException
     */
    public function get(string $key): mixed {
        if (!array_key_exists($key, $this->data)) {
            throw new \InvalidArgumentException("Unknown key: $key");
        }
        return $this->data[$key];
    }
}
===file:Main.php===
<?php
// Extend triggers PSR-4 lazy-load of App\Config, which throws a stdlib exception
class AppConfig extends \App\Config {}

$config = new AppConfig(['debug' => true]);
try {
    $val = $config->get('debug');
} catch (\InvalidArgumentException $e) {
    echo $e->getMessage();
}
===expect===
