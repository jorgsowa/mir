===description===
Absolute parameter types resolve the exact class despite short-name collisions.
===file:composer.json===
{
  "autoload": {
    "psr-4": {
      "Repro\\": "src/",
      "Alpha\\": "vendor/pkg/decoy/src/",
      "Zeta\\": "vendor/pkg/target/src/"
    }
  }
}
===file:vendor/pkg/target/src/Widget.php===
<?php
namespace Zeta;
class Widget {
    public function targetOnly(): void {}
}
===file:vendor/pkg/decoy/src/Widget.php===
<?php
namespace Alpha;
class Widget {
    public function decoyOnly(): void {}
}
===file:Main.php===
<?php
namespace Repro;
class Main {
    public function run(\Zeta\Widget $widget): void {
        $widget->targetOnly();
    }
}
===expect===
