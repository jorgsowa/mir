===description===
a private static method used only as a "Class::method" string argument to a user function with a plain `callable` param must not be reported unused
===config===
suppress=
===file===
<?php
function invokeCallback(callable $cb): void {
    $cb();
}

class Handler {
    private static function handle(): void {
        echo "here";
    }

    public static function run(): void {
        invokeCallback('Handler::handle');
    }
}

Handler::run();
===expect===
