===description===
A closure literal rebound with a non-literal scope (variable, object, omitted)
or via call() gets the new object as $this, so its body must not be checked
against the enclosing class.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Target {
    private function hidden(): int {
        return 1;
    }
}
class Host {
    public function assignThenBindTo(Target $t, string $scope): void {
        $f = function () { return $this->hidden(); };
        $f->bindTo($t, $scope);
    }
    public function scopeIsObject(Target $t): void {
        $f = function () { return $this->hidden(); };
        $f->bindTo($t, $t);
    }
    public function scopeOmitted(Target $t): void {
        $f = function () { return $this->hidden(); };
        $f->bindTo($t);
    }
    public function inlineBindTo(Target $t, string $scope) {
        return (function () { return $this->hidden(); })->bindTo($t, $scope);
    }
    public function inlineLiteralScope(Target $t) {
        return (function () { return $this->hidden(); })->bindTo($t, Target::class);
    }
    public function staticBind(Target $t, string $scope) {
        return Closure::bind(function () { return $this->hidden(); }, $t, $scope);
    }
    public function assignThenStaticBind(Target $t, string $scope) {
        $f = function () { return $this->hidden(); };
        return Closure::bind($f, $t, $scope);
    }
    public function callForm(Target $t): void {
        $f = function () { return $this->hidden(); };
        $f->call($t);
    }
    public function inlineCall(Target $t): void {
        (function () { return $this->hidden(); })->call($t);
    }
    public function arrowForm(Target $t, string $scope) {
        return (fn() => $this->hidden())->bindTo($t, $scope);
    }
}
