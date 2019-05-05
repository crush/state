# state

Often when designing software solutions, one would like for applications to
offer a degree of robustness in the face of unexpected errors/crashes.  If
an application could represent its entire state as a single JSON object,
then that application could be made fault-tolerant by:

1. Periodically recording the application state,
2. Restarting the application with the last recorded stat as input and
3. Watching the application process for crashes and handling restarts.

State exists to do exactly this.

## Application Interface

When state starts an application, it will supply the last recorded state
as a single command-line argument as a string.  To record new states,
the `stdout` of the application state manages will be interpreted as a
stream of JSON objects.  Finally, state will redirect `stdin` to the
process and `stderr` to the console.

The following application satisfies state's interface and could thus be
run by state to count the number of times the program is started.

**example1.py**

```py
#! /usr/bin/env python

import json
import sys

def main():
    state = json.loads(sys.argv[1]).get('state', {})
    count = state.get('count', 0)

    state['count'] = count + 1
    print(json.dumps(state))


if __name__ == '__main__':
    main()
```

When run with `state run example1.py --file .state.json`, state will run
the application and log its state to a file called `.state.json`.
