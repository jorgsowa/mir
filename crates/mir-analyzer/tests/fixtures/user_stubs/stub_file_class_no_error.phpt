===config===
stub_file=stubs/framework.php
===file:stubs/framework.php===
<?php
class FrameworkClient {
    public function connect(string $url): bool { return true; }
}
===file:App.php===
<?php
function boot(): void {
    $client = new FrameworkClient();
    $client->connect('https://example.com');
}
===expect===
