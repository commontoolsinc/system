import { read, write } from 'common:io/state@0.0.1';

export class Body {
    run() {
        const foo = read('foo');
        const value = foo?.deref();

        write('bar', {
          tag: 'string',
          val: 'baz'
        });
    }
}

export const module = {
  Body,

  create() {
      return new Body();
  }
};