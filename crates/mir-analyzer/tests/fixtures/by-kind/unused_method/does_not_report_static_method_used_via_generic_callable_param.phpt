===description===
a static method used only as a "Class::method" string argument to a user function with a plain `callable` param must not be reported unused
===file===
<?php
class Handler {
    public static function handle(): void {
        echo "here";
    }
}

function invokeCallback(callable $cb): void {
    $cb();
}

invokeCallback('Handler::handle');
===expect===
