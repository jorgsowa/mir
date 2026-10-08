===description===
Control for dynamic-scope rebinding: only a rebind to another object makes
$this unknown. A closure that is never rebound, is rebound to $this, is
unbound with null, or whose rebind is not adjacent is still checked
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
}
class Host {
    public function ours(): void {}

    public function notRebound() {
        $f = function () { return $this->missing(); };
//                                ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Host::missing() does not exist
        return $f;
    }
    public function reboundToThis(string $scope) {
        $f = function () { return $this->missing(); };
//                                ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Host::missing() does not exist
        return $f->bindTo($this, $scope);
    }
    public function unbound(string $scope) {
        $f = function () { return $this->missing(); };
//                                ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Host::missing() does not exist
        return $f->bindTo(null, $scope);
    }
    public function rebindNotAdjacent(Target $t, string $scope) {
        $f = function () { return $this->missing(); };
//                                ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Host::missing() does not exist
        $this->ours();
        return $f->bindTo($t, $scope);
    }
    public function nestedClosureKeepsEnclosing(Target $t, string $scope) {
        return (function () {
            return function () { return $this->missing(); };
//                                      ^^^^^^^^^^^^^^^^ UndefinedMethod: Method Host::missing() does not exist
        })->bindTo($t, $scope);
    }
}
