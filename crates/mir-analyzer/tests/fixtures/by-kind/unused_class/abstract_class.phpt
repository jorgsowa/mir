===description===
An abstract class nothing extends or references is unused; one that is extended is not
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
abstract class NeverExtended {
//       ^ +2:1 UnusedClass: Class NeverExtended is never referenced
    abstract public function work(): void;
}

abstract class Extended {
    abstract public function work(): void;
}

final class Worker extends Extended {
    public function work(): void {}
}

(new Worker())->work();
