===description===
FP guard: non-template function returning same-namespace class must not trigger assignment error
===file:Lib/Widget.php===
<?php
namespace Lib;

class Widget {}

function makeWidget(): Widget {
    return new Widget();
}
===file:App/Consumer.php===
<?php
namespace App;

use Lib\Widget;

class Consumer {
    public Widget $w;

    public function run(): void {
        $this->w = \Lib\makeWidget();
    }
}
===expect===
