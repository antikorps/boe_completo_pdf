BOE Completo en PDF (CLI)
=============================

Este programa permite descargar y combinar los documentos PDF del **Boletín Oficial del Estado (BOE)** de España para una fecha específica, todo a través de la línea de comandos.

Instalación
-----------

1.  **Descargar el ejecutable**:
    
    *   Puedes descargar el ejecutable correspondiente a tu sistema operativo desde la sección **Releases** del repositorio.
    *   Otorga permisos de ejecución si tu sistema operativo lo exige.

Uso
---

Una vez que tengas el ejecutable disponible, puedes usar el gestor de descargas desde la línea de comandos de la siguiente manera:

### Comando

```bash
./gestor_boe <fecha>
```

Donde:

*   `<fecha>` es la fecha en formato `DD-MM-YYYY` de los documentos que deseas descargar. Ejemplo: `07-12-2024`.

### Ejemplo

Si deseas descargar los documentos del BOE correspondientes al **7 de diciembre de 2024**, ejecuta:

```bash
./gestor_boe 07-12-2024
```

El programa buscará los enlaces a los PDFs de disposiciones, notificaciones y edictos del día especificado, los descargará, los combinará en un solo PDF y generará un informe con los enlaces.

### Cambiar el tiempo de espera entre descargas

El tiempo de espera entre cada descarga de los archivos PDF se puede configurar mediante la variable de entorno `BOE_COMPLETO_ESPERA`. Esta variable define el tiempo (en segundos) que el programa espera entre descargas para evitar sobrecargar el servidor del BOE.

**Valor predeterminado**: 3 segundos.

Puedes modificar el valor de esta variable antes de ejecutar el programa:

```bash
export BOE_COMPLETO_ESPERA=5
```
**Importante**: **No se recomienda reducir el tiempo de espera**, ya que hacer peticiones demasiado rápidas puede llevar a que el servidor del BOE bloquee el acceso del programa.

Salida
------

*   **Éxito**: Si todo va bien, el programa combinará los archivos PDF y los guardará en el directorio donde se encuentra el ejecutable.
*   **Error**: Si ocurre algún error (como problemas en la descarga o en la generación del PDF), el programa saldrá con un código de error (`exit(1)`).

Notas
-----

*   El archivo PDF combinado será guardado en el mismo directorio con un nombre basado en la fecha de la descarga.
*   Se generará también un archivo de informe en formato `.tsv` con los enlaces a los PDFs descargados.
*   El programa no requiere configuración adicional, más allá de la variable de entorno para el tiempo de espera si deseas modificarlo.

Aviso importante
----------------
El desarrollo de esta utilidad ha sido un mero entretenimiento. No soy usuario activo de la página del BOE, he cogido una fecha y he asumido que todos los días serán exactamente como ese, lo cual es bastante atrevido.

Tampoco he realizado ningún tipo de comprobación del archivo generado, asumiendo, entre otras cuestiones, que el orden generado es correcto.

Si alguien tiene especial interés en utilizar esta herramienta le recomiendo que antes de nada revise los [documentos generados al procesar la fecha de ejemplo (07-12-2024)](https://easyupload.io/b34v89). Para ello puede comparar el archivo **07_12_2024_boe_completo.pdf**, con **07_12_2024_boe_completo_informe.tsv** y con la información disponible en el [enlace oficial](https://boe.es/boe/dias/2024/12/07/)