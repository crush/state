#! /usr/bin/env python

import json
import sys
import time


def main():
    state = json.loads(sys.argv[1]).get('state', {})
    count = state.get('count', 0)

    state['count'] = count + 1
    print(json.dumps(state))
    time.sleep(2)


if __name__ == '__main__':
    main()
