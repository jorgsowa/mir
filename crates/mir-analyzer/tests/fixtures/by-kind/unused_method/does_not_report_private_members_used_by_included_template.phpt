===description===
A file included from a method runs in its scope, so it can use any private member through `$this`
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Renderer {
    private string $title = 'x';

    private function escape(string $s): string { return htmlspecialchars($s); }

    public function render(string $template): void {
        include $template;
    }
}

(new Renderer())->render('page.html.php');
