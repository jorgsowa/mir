===description===
stub file class no error
===config===
<mir>
  <stubs>
    <file name="stubs/framework.php"/>
  </stubs>
</mir>
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
