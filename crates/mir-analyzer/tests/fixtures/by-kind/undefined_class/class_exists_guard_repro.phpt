===description===
repro: class_exists guard does not suppress UndefinedClass on new in true branch
===file===
<?php
function test(): void {
    if (class_exists(\Pusher\Pusher::class)) {
        new \Pusher\Pusher('key', 'secret', 'app_id');
    }
}
===expect===
