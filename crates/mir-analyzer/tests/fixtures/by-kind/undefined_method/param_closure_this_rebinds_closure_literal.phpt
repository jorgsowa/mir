===description===
`@param-closure-this` rebinds `$this` (and scope) inside a closure literal passed for that
parameter: static call, instance call, free function, named class, and the no-tag control.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Widget {
    public function render(): string { return ''; }
    private function secret(): int { return 1; }
}
class Registry {
    /** @param-closure-this static $fn */
    public static function define(string $name, callable $fn): void {}

    /** @param-closure-this Widget $fn */
    public function extend(callable $fn): void {}

    /** @param-closure-this static $fn */
    public function macro(callable $fn): void {}

    public function onlyTagged(string $name, callable $plain, callable $fn): void {}

    public function ownMethod(): int { return 1; }
}
class SubRegistry extends Registry {
    public function subOnly(): int { return 2; }
}
/** @param-closure-this Widget $fn */
function with_widget(callable $fn): void {}

class Consumer {
    public function viaStatic(): void {
        Registry::define('a', function () {
            /** @mir-check $this is Registry */
            return $this->ownMethod();
        });
        SubRegistry::define('b', function () {
            /** @mir-check $this is SubRegistry */
            return $this->subOnly();
        });
    }
    public function viaInstance(Registry $r, SubRegistry $s): void {
        $r->extend(function () {
            return $this->render();
        });
        $s->macro(function () {
            return $this->subOnly();
        });
        $r->macro(fn() => $this->ownMethod());
    }
    public function viaFunction(): void {
        with_widget(function () {
            return $this->secret();
        });
        with_widget(fn() => $this->render());
    }
    public function untaggedParamKeepsEnclosingClass(Registry $r): void {
        $r->onlyTagged('x', function () {
            return $this->viaStatic();
        }, function () {
            return $this->render();
//                 ^^^^^^^^^^^^^^^ UndefinedMethod: Method Consumer::render() does not exist
        });
    }
    public function missingMethodStillReported(Registry $r): void {
        $r->extend(function () {
            return $this->nope();
//                 ^^^^^^^^^^^^^ UndefinedMethod: Method Widget::nope() does not exist
        });
    }
}
