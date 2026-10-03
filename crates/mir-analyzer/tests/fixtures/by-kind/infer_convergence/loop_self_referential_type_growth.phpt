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
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class NotificationSender
{
    protected $locale;
    public $bus;

    protected function queueNotification($notifiables, $notification)
    {
        $original = clone $notification;
//                  ^^^^^^^^^^^^^^^^^^^ MixedClone: cannot clone mixed

        foreach ($notifiables as $notifiable) {
            foreach ((array) $original->via($notifiable) as $channel) {
//                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^ MixedMethodCall: Method via() called on mixed type
                $notification = clone $original;
//                              ^^^^^^^^^^^^^^^ MixedClone: cannot clone mixed

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
//              ^ +6:17 MixedMethodCall: Method dispatch() called on mixed type
                    (new SendQueuedNotifications($notifiable, $notification, [$channel]))
//                  ^ +2:41 MixedMethodCall: Method onQueue() called on mixed type
//                       ^^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class SendQueuedNotifications does not exist
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
