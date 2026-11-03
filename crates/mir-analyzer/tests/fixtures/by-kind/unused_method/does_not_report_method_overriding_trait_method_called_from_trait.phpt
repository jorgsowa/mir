===description===
A trait body calling its own private method reaches the composing class's override
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
trait BuildsRoutes {
    public function add(string $path): string {
        return $this->createRoute($path);
    }

    private function createRoute(string $path): string {
        return $path;
    }
}

final class Collection {
    use BuildsRoutes;

    private function createRoute(string $path): string {
        return '/prefix' . $path;
    }
}

final class Other {
    use BuildsRoutes;

    private function unrelated(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Other::unrelated() is never called

    public function go(): string {
        return $this->createRoute('x');
    }
}

(new Collection())->add('/a');
(new Other())->go();
