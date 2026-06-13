===description===
Regression: several method_exists-guarded reassignments on the SAME variable,
each of the form `$v = $x->method()[$key] ?? null`, inside nested foreach loops.
This is the reduced form of Laravel's
Illuminate\Notifications\NotificationSender::queueNotification. The bug:
FlowState::merge_branches concatenated each branch's dead-write Vec onto a
pre-cloned copy, and since both branches descend from pre, every merge roughly
tripled the dead-write set; under nested-loop fixpoint analysis this grew
exponentially (3^N in the number of sequential ifs), allocating ~20 GB and
OOM-ing on this single file (v0.31/0.32 regression; v0.30 never exhibited it).
Fixed by deduplicating dead writes when merging branches. This fixture guards
the fix: with the bug it OOMs; fixed, it completes in milliseconds. Diagnostics
are identical either way (the blow-up is transient), so the guard is that it
analyzes at all.
===config===
suppress=MissingPropertyType,MixedArgument,MixedArrayAccess,MixedArrayOffset,MixedAssignment,MixedPropertyFetch
===file===
<?php
class NotificationSender
{
    protected $locale;
    public $bus;

    protected function queueNotification($notifiables, $notification)
    {
        $original = clone $notification;

        foreach ($notifiables as $notifiable) {
            foreach ((array) $original->via($notifiable) as $channel) {
                $notification = clone $original;

                $connection = $notification->connection;
                if (method_exists($notification, 'viaConnections')) {
                    $connection = $notification->viaConnections()[$channel] ?? null;
                }

                $queue = $notification->queue;
                if (method_exists($notification, 'viaQueues')) {
                    $queue = $notification->viaQueues()[$channel] ?? null;
                }

                $delay = $notification->delay;
                if (method_exists($notification, 'withDelay')) {
                    $delay = $notification->withDelay($notifiable, $channel) ?? null;
                }

                $middleware = $notification->middleware ?? [];
                if (method_exists($notification, 'middleware')) {
                    $middleware = array_merge(
                        $notification->middleware($notifiable, $channel),
                        $middleware
                    );
                }

                $this->bus->dispatch(
                    (new SendQueuedNotifications($notifiable, $notification, [$channel]))
                        ->onConnection($connection)
                        ->onQueue($queue)
                        ->delay(is_array($delay) ? ($delay[$channel] ?? null) : $delay)
                        ->through($middleware)
                );
            }
        }
    }
}
===expect===
MixedClone@9:21-9:40: cannot clone mixed
MixedMethodCall@12:30-12:57: Method via() called on mixed type
MixedClone@13:33-13:48: cannot clone mixed
MixedMethodCall@38:17-44:18: Method dispatch() called on mixed type
MixedMethodCall@39:21-41:42: Method onQueue() called on mixed type
UndefinedClass@39:26-39:49: Class SendQueuedNotifications does not exist
