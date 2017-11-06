import Config from './config'
import KeyBind from './keybind'
import Menu from './menu'
import PackageManager from './package-manager'
import PackageRegistry from './package-registry'
import minimist from 'minimist'

const fields = Symbol('fields')
export default class Deplug {
  constructor (argv) {
    const options = minimist(argv)
    const config = new Config(options.profile, 'config')
    this[fields] = {
      config,
      layout: new Config(options.profile, 'layout'),
      keybind: new KeyBind(options.profile),
      packages: new PackageManager(config),
      registry: new PackageRegistry(config),
      menu: new Menu(),
      argv,
    }
  }

  get config () {
    return this[fields].config
  }

  get layout () {
    return this[fields].layout
  }

  get keybind () {
    return this[fields].keybind
  }

  get menu () {
    return this[fields].menu
  }

  get packages () {
    return this[fields].packages
  }

  get registry () {
    return this[fields].registry
  }

  get argv () {
    return this[fields].argv
  }
}