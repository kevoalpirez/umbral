# Umbral · aplicación privada de notas

Umbral es una aplicación de escritorio basada en Tauri. Se puede compilar para macOS, Windows y Linux. Incluye perfiles locales: cada usuario tiene una bóveda independiente y cifrada con su propia contraseña.

## Qué incluye

Cuadernos y páginas anidadas, enlaces entre páginas, tareas con fechas, búsqueda, favoritas, papelera recuperable, duplicación, tarjetas de estudio, guardado automático, bloqueo manual y por inactividad, respaldo cifrado y exportación Markdown. Todo ocurre dentro de los cuadernos. No incluye colaboración ni sincronización entre dispositivos.

## Instalar herramientas para desarrollo

Instala Node.js LTS y Rust. Tauri requiere herramientas adicionales según el sistema operativo; consulta sus requisitos vigentes en [Tauri: prerequisitos](https://v2.tauri.app/start/prerequisites/).

Desde la carpeta `umbral`:

```sh
npm install
npm run desktop
```

Esto abre Umbral como una ventana nativa en modo de desarrollo. No se necesita ejecutar `python3 -m http.server` para usar la aplicación de escritorio.

## Generar paquetes instalables

```sh
npm run build
```

Tauri genera el paquete compatible con el sistema en que se ejecuta la compilación, bajo `src-tauri/target/release/bundle/`. En general:

- macOS: `.app` y `.dmg`
- Windows: instalador `.msi` o `.exe`
- Linux: `.deb`, `.rpm` y/o `.AppImage`, según las herramientas disponibles

Para distribuir a personas en los tres sistemas, compila en cada sistema (o configura una compilación automatizada para macOS, Windows y Linux). La firma y notarización de macOS requieren certificados de Apple; Windows también puede mostrar avisos si el instalador no está firmado. Los paquetes generados aquí no se firman.

En Mac, `npm run desktop` solo abre la versión de desarrollo; no instala Umbral en Aplicaciones. Para crear la app, ejecuta `npm run build`. Después abre `src-tauri/target/release/bundle/dmg/`, abre el archivo `.dmg` generado y arrastra Umbral a la carpeta Aplicaciones.

## Usuarios, privacidad y almacenamiento

Los usuarios son **perfiles locales**, no cuentas en línea. Crear un usuario no envía datos a un servidor; no hace falta administrar un servidor propio. Los perfiles pertenecen a la cuenta de sistema operativo que ejecuta Umbral y no se sincronizan automáticamente con otro equipo o teléfono. Para usar los mismos datos en varios dispositivos se necesitaría añadir una función de sincronización, que este proyecto todavía no incluye.

Cada bóveda se cifra con AES-GCM de 256 bits y una clave derivada de la contraseña mediante PBKDF2-SHA256. Las contraseñas no se guardan. El registro local solo contiene los nombres e identificadores de los perfiles; el contenido permanece en archivos cifrados dentro del directorio de datos de la aplicación. La bóveda anterior `workspace.umbral` se conserva como el perfil “Espacio anterior” para que sus datos sigan disponibles.

La prueba web opcional guarda los perfiles en el almacenamiento del navegador. Para trabajar con la app nativa, los cambios se guardan en el directorio de datos privado de Umbral. Descarga respaldos cifrados con regularidad: si pierdes la contraseña, no se puede recuperar la bóveda. La exportación Markdown no está cifrada. El cifrado en reposo no protege frente a un dispositivo comprometido o una aplicación abierta.

## Probar en navegador (opcional)

```sh
python3 -m http.server 1420 --bind 127.0.0.1 --directory src
```

Abre http://localhost:1420. Esta modalidad es solo una vista previa local y no comparte automáticamente sus datos con la aplicación nativa.
