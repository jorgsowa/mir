===description===
bracketed namespace class not reported
===file===
<?php
namespace MyApp {
    class MyService {}

    function test(): void {
        new MyService();
    }
}
===expect===
